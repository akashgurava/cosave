//! Tier 2 direct database integration and referential integrity tests for families, members, and accounts.
//!
//! Verifies database engine behaviors against an isolated in-memory SQLite pool:
//! - Foreign key cascading deletions (`ON DELETE CASCADE`) from family to members and accounts.
//! - Cascading account purges when individual members are deleted.
//! - Scope-enforced uniqueness constraints (`UNIQUE(family_id, member_name)`, `UNIQUE(owner_member_id, account_name)`).
//! - Foreign key validation on account creation with invalid member IDs.

use crate::core::{init_db, is_foreign_key_violation, AppConfig, DbPool};

use super::super::models::{CurrencyCode, FamilyName};
use super::*;

async fn setup_test_db() -> DbPool {
    let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
        .await
        .expect("init test sqlite in-memory db");
    let mut tx = pool.begin().await.expect("begin tx");
    init_family_schema(&mut tx)
        .await
        .expect("init family schema");
    tx.commit().await.expect("commit tx");
    seed_default_family(&pool)
        .await
        .expect("seed default family");
    pool
}

#[tokio::test]
async fn test_cascade_delete_member_removes_accounts() {
    let pool = setup_test_db().await;

    let (sarah_id,): (i64,) =
        sqlx::query_as("SELECT id FROM members WHERE member_name = 'Sarah Miller';")
            .fetch_one(&pool)
            .await
            .unwrap();

    let (acc_before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE owner_member_id = ?;")
            .bind(sarah_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(acc_before, 2, "Sarah should initially own 2 accounts");

    // Delete member via db function
    delete_member(&pool, sarah_id).await.unwrap();

    let (acc_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE owner_member_id = ?;")
            .bind(sarah_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        acc_after, 0,
        "accounts must be cascade deleted when owner member is deleted"
    );
}

#[tokio::test]
async fn test_cascade_delete_family_removes_members_and_accounts() {
    let pool = setup_test_db().await;

    let (family_id,): (i64,) = sqlx::query_as("SELECT id FROM families LIMIT 1;")
        .fetch_one(&pool)
        .await
        .unwrap();

    let (members_before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM members WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(members_before, 3);

    let (accounts_before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(accounts_before, 4);

    sqlx::query("DELETE FROM families WHERE id = ?;")
        .bind(family_id)
        .execute(&pool)
        .await
        .unwrap();

    let (members_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM members WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(members_after, 0, "all members cascade deleted with family");

    let (accounts_after,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        accounts_after, 0,
        "all accounts cascade deleted with family"
    );
}

#[tokio::test]
async fn test_foreign_key_invalid_owner_fails() {
    let pool = setup_test_db().await;

    let res = sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (1, 999999, 'bank_account', 'INR', 'Test Bank', '1234', 'Checking', 1000, 0, 0);
        "#,
    )
    .execute(&pool)
    .await;

    assert!(
        res.is_err(),
        "inserting account with non-existent owner must violate FK constraint"
    );
    let err = res.unwrap_err();
    assert!(is_foreign_key_violation(&err));
}

#[tokio::test]
async fn test_seed_default_family_idempotent() {
    let pool = setup_test_db().await;

    // Call seed again
    seed_default_family(&pool)
        .await
        .expect("second seed call succeeds without error");

    let (family_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM families;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        family_count, 1,
        "families table remains with single household"
    );

    let (member_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM members;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(member_count, 3, "members count does not duplicate");

    let (account_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(account_count, 4, "accounts count does not duplicate");
}

#[tokio::test]
async fn test_get_default_currency_reflects_family_and_updates() {
    let pool = setup_test_db().await;

    let currency = get_default_currency(&pool, None).await.unwrap();
    assert_eq!(currency, "INR");

    let gbp = CurrencyCode::try_new("GBP", "TEST").unwrap();
    let name = FamilyName::try_new("The British Millers", "TEST").unwrap();
    update_family(&pool, Some(name), Some(gbp)).await.unwrap();

    let updated_currency = get_default_currency(&pool, None).await.unwrap();
    assert_eq!(updated_currency, "GBP");
}

#[tokio::test]
async fn test_family_schema_unique_constraint_violations() {
    let pool = setup_test_db().await;

    // 1. Family name unique constraint
    let dup_family_err = sqlx::query(
        "INSERT INTO families (family_name, currency, created_at, updated_at) VALUES ('The Miller Family', 'USD', 0, 0);",
    )
    .execute(&pool)
    .await
    .expect_err("duplicate family name must fail");
    assert!(crate::core::is_unique_violation(&dup_family_err));

    // 2. Member name unique within the same family
    let (family_id,): (i64,) = sqlx::query_as("SELECT id FROM families LIMIT 1;")
        .fetch_one(&pool)
        .await
        .unwrap();

    let dup_member_err = sqlx::query(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'Sarah Miller', 0, 0);",
    )
    .bind(family_id)
    .execute(&pool)
    .await
    .expect_err("duplicate member name in same family must fail");
    assert!(crate::core::is_unique_violation(&dup_member_err));

    // 3. Member with same name in a DIFFERENT family must succeed
    let other_family_id: i64 = sqlx::query_scalar(
        "INSERT INTO families (family_name, currency, created_at, updated_at) VALUES ('The Jones Family', 'EUR', 0, 0) RETURNING id;",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let diff_family_member = sqlx::query(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'Sarah Miller', 0, 0);",
    )
    .bind(other_family_id)
    .execute(&pool)
    .await;
    assert!(
        diff_family_member.is_ok(),
        "same member name in different family is allowed"
    );

    // 4. Account name unique for a member (Sarah already has 'Total Checking')
    let (sarah_id,): (i64,) =
        sqlx::query_as("SELECT id FROM members WHERE member_name = 'Sarah Miller';")
            .fetch_one(&pool)
            .await
            .unwrap();

    let dup_account_err = sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', 'INR', 'Chase', '9999', 'Total Checking', 1000, 0, 0);
        "#,
    )
    .bind(family_id)
    .bind(sarah_id)
    .execute(&pool)
    .await
    .expect_err("duplicate account name for same member must fail");
    assert!(crate::core::is_unique_violation(&dup_account_err));

    // 5. Account with same name for a DIFFERENT member must succeed (David also having 'Total Checking')
    let (david_id,): (i64,) =
        sqlx::query_as("SELECT id FROM members WHERE member_name = 'David Miller';")
            .fetch_one(&pool)
            .await
            .unwrap();

    let diff_member_acc = sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', 'INR', 'Chase', '8888', 'Total Checking', 2000, 0, 0);
        "#,
    )
    .bind(family_id)
    .bind(david_id)
    .execute(&pool)
    .await;
    assert!(
        diff_member_acc.is_ok(),
        "same account name for different member is allowed"
    );
}
