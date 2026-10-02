//! Database queries and atomic mutations for family members.
//!
//! Manages member roster persistence and lifecycle:
//! - **Single-Shot Insert Operations**: Validates [`MemberName`] and inserts via atomic `INSERT ... RETURNING id`.
//! - **Constraint Classification**: Translates SQLite constraint violations (`UNIQUE(family_id, member_name)`,
//!   `FOREIGN KEY(family_id)`) to domain [`FamilyError`] variants without transaction overhead.
//! - **Cascading Deletions**: Deletes member records while database foreign key triggers automatically
//!   cascade removal to all owned accounts.

use sqlx::Row;

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};

use super::super::error::FamilyError;
use super::super::models::{CreateMemberRequest, MemberDto, MemberName, UpdateMemberRequest};

/// Creates a new family member under the active family as a single atomic INSERT operation.
///
/// Validates the member name Value Object, inserts into `members` within the specified
/// household family, and returns the newly created member entity.
///
/// # Execution Model
/// Executes a single atomic `INSERT ... RETURNING id` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`]
/// (`UNIQUE(family_id, member_name)`) and [`is_foreign_key_violation`] (`FOREIGN KEY(family_id)`).
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateMemberRequest`] containing target family ID and member name.
///
/// # Returns
/// - `Ok(MemberDto)` representing the newly created member with generated ID.
///
/// # Errors
/// - Returns [`FamilyError::EmptyMemberName`] if member name fails Value Object validation.
/// - Returns [`FamilyError::MemberAlreadyExists`] if a member with this name already exists in this family.
/// - Returns [`FamilyError::FamilyNotFound`] if the parent family does not exist.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn create_member(
    pool: &DbPool,
    payload: CreateMemberRequest,
) -> Result<MemberDto, AppError> {
    let name = MemberName::try_new(payload.member_name(), "FAMILY.CREATE_MEMBER.EMPTY_NAME")?;
    let family_id = payload.family_id();
    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO members (family_id, member_name, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        RETURNING id;
        "#,
    )
    .bind(family_id)
    .bind(&raw_name)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    match res {
        Ok(id) => Ok(MemberDto::new(id, family_id, raw_name, now)),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::MemberAlreadyExists {
                    action: "FAMILY.CREATE_MEMBER.ALREADY_EXISTS",
                    member_name: raw_name,
                }
                .into())
            } else if is_foreign_key_violation(&err) {
                Err(FamilyError::FamilyNotFound {
                    action: "FAMILY.CREATE_MEMBER.FAMILY_NOT_FOUND",
                }
                .into())
            } else {
                Err(db_err("FAMILY.CREATE_MEMBER.INSERT", err))
            }
        }
    }
}

/// Updates an existing member's display name as a single atomic UPDATE operation.
///
/// Validates the new name Value Object and persists changes using `RETURNING` to fetch
/// the updated entity without a separate subsequent query.
///
/// # Execution Model
/// Executes a single atomic `UPDATE ... RETURNING` directly against [`DbPool`].
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`].
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the target member.
/// - `payload`: Inbound [`UpdateMemberRequest`] with new member name.
///
/// # Returns
/// - `Ok(MemberDto)` representing the updated member entity.
///
/// # Errors
/// - Returns [`FamilyError::EmptyMemberName`] if member name fails Value Object validation.
/// - Returns [`FamilyError::MemberNotFound`] if the target member ID does not exist.
/// - Returns [`FamilyError::MemberAlreadyExists`] if the new name collides with another member in the same family.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn update_member(
    pool: &DbPool,
    id: i64,
    payload: UpdateMemberRequest,
) -> Result<MemberDto, AppError> {
    let name = MemberName::try_new(payload.member_name(), "FAMILY.UPDATE_MEMBER.EMPTY_NAME")?;
    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query(
        r#"
        UPDATE members
        SET member_name = ?, updated_at = ?
        WHERE id = ?
        RETURNING id, family_id, member_name, created_at;
        "#,
    )
    .bind(&raw_name)
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await;

    match res {
        Ok(Some(r)) => Ok(MemberDto::new(
            r.get("id"),
            r.get("family_id"),
            r.get::<String, _>("member_name"),
            r.get("created_at"),
        )),
        Ok(None) => Err(FamilyError::MemberNotFound {
            action: "FAMILY.UPDATE_MEMBER.NOT_FOUND",
            id,
        }
        .into()),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(FamilyError::MemberAlreadyExists {
                    action: "FAMILY.UPDATE_MEMBER.ALREADY_EXISTS",
                    member_name: raw_name,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_MEMBER.EXECUTE", err))
            }
        }
    }
}

/// Deletes a family member by ID, cascading removal of all owned accounts.
///
/// Removes the member entity from `members`. Configured SQLite foreign key cascades
/// (`ON DELETE CASCADE`) automatically purge all depository and revolving accounts owned
/// by this member.
///
/// # Execution Model
/// Executes a single atomic `DELETE FROM members WHERE id = ?` directly against [`DbPool`],
/// validating that at least one row was affected.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `id`: 64-bit integer identifier of the member to delete.
///
/// # Returns
/// - `Ok(())` on successful deletion.
///
/// # Errors
/// - Returns [`FamilyError::MemberNotFound`] if no member matching `id` was found.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(crate) async fn delete_member(pool: &DbPool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM members WHERE id = ?;")
        .bind(id)
        .execute(pool)
        .await
        .db_context("FAMILY.DELETE_MEMBER.EXECUTE")?;

    if result.rows_affected() == 0 {
        return Err(FamilyError::MemberNotFound {
            action: "FAMILY.DELETE_MEMBER.NOT_FOUND",
            id,
        }
        .into());
    }

    Ok(())
}
