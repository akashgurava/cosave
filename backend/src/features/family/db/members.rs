use sqlx::Row;

use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
    DbResultExt,
};

use super::super::error::FamilyError;
use super::super::models::{CreateMemberRequest, MemberDto, MemberName, UpdateMemberRequest};

/// Creates a new family member under the active family as a single atomic INSERT operation.
pub(crate) async fn create_member(
    pool: &DbPool,
    payload: CreateMemberRequest,
) -> Result<MemberDto, AppError> {
    let name = MemberName::try_new(payload.name(), "FAMILY.CREATE_MEMBER.EMPTY_NAME")?;
    let family_id = payload.family_id();
    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO members (family_id, name, created_at, updated_at)
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
                    name: raw_name,
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
pub(crate) async fn update_member(
    pool: &DbPool,
    id: i64,
    payload: UpdateMemberRequest,
) -> Result<MemberDto, AppError> {
    let name = MemberName::try_new(payload.name(), "FAMILY.UPDATE_MEMBER.EMPTY_NAME")?;
    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query(
        r#"
        UPDATE members
        SET name = ?, updated_at = ?
        WHERE id = ?
        RETURNING id, family_id, name, created_at;
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
            r.get::<String, _>("name"),
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
                    name: raw_name,
                }
                .into())
            } else {
                Err(db_err("FAMILY.UPDATE_MEMBER.EXECUTE", err))
            }
        }
    }
}

/// Deletes a family member by ID, cascading removal of all owned accounts.
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
