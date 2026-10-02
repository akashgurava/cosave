//! Tier 2 direct database integration and referential integrity tests for families, members, and accounts.
//!
//! Verifies database engine behaviors against an isolated in-memory SQLite pool:
//! - Foreign key cascading deletions (`ON DELETE CASCADE`) from family to members and accounts.
//! - Cascading account purges when individual members are deleted.
//! - Scope-enforced uniqueness constraints (`UNIQUE(family_id, member_name)`, `UNIQUE(owner_member_id, account_name)`).
//! - Foreign key validation on account creation with invalid member IDs.

use crate::core::{init_db, is_foreign_key_violation, AppConfig, DbPool};

use super::super::models::FamilyName;
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

    // Explicitly insert family, Sarah, and accounts
    sqlx::query(
        "INSERT INTO families (id, family_name, currency_id, created_at, updated_at) VALUES (1, 'Test Family', 1, 0, 0);",
    )
    .execute(&pool)
    .await
    .unwrap();

    let sarah_id: i64 = sqlx::query_scalar(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (1, 'Sarah Miller', 0, 0) RETURNING id;",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO accounts (family_id, owner_member_id, type, currency_id, bank_name, last4, account_name, available_balance_cents, created_at, updated_at) VALUES (1, ?, 'bank_account', 1, 'Chase', '4821', 'Checking', 1000, 0, 0);",
    )
    .bind(sarah_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO accounts (family_id, owner_member_id, type, currency_id, bank_name, last4, account_name, credit_limit_cents, available_cents, created_at, updated_at) VALUES (1, ?, 'credit_card', 1, 'Chase', '5561', 'Card', 2000, 1000, 0, 0);",
    )
    .bind(sarah_id)
    .execute(&pool)
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

    let family_id: i64 = sqlx::query_scalar(
        "INSERT INTO families (family_name, currency_id, created_at, updated_at) VALUES ('Cascade Family', 1, 0, 0) RETURNING id;",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Insert 2 members and 2 accounts
    let m1: i64 = sqlx::query_scalar(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'M1', 0, 0) RETURNING id;",
    )
    .bind(family_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let m2: i64 = sqlx::query_scalar(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'M2', 0, 0) RETURNING id;",
    )
    .bind(family_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO accounts (family_id, owner_member_id, type, currency_id, bank_name, last4, account_name, available_balance_cents, created_at, updated_at) VALUES (?, ?, 'bank_account', 1, 'Chase', '1111', 'A1', 1000, 0, 0);",
    )
    .bind(family_id)
    .bind(m1)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO accounts (family_id, owner_member_id, type, currency_id, bank_name, last4, account_name, available_balance_cents, created_at, updated_at) VALUES (?, ?, 'bank_account', 1, 'Chase', '2222', 'A2', 2000, 0, 0);",
    )
    .bind(family_id)
    .bind(m2)
    .execute(&pool)
    .await
    .unwrap();

    let (members_before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM members WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(members_before, 2);

    let (accounts_before,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE family_id = ?;")
            .bind(family_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(accounts_before, 2);

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
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (1, 999999, 'bank_account', 1, 'Test Bank', '1234', 'Checking', 1000, 0, 0);
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
        family_count, 0,
        "families table remains blank after seeding"
    );

    let (member_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM members;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(member_count, 0, "members start blank");

    let (account_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(account_count, 0, "accounts start blank");

    let (curr_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM currencies;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(curr_count, 20, "currencies loaded from json");
}

#[tokio::test]
async fn test_get_default_currency_reflects_family_and_updates() {
    let pool = setup_test_db().await;

    // Regional resolution checks prior to family creation
    let currency_in = get_default_currency(&pool, Some("IN")).await.unwrap();
    assert_eq!(currency_in, "INR");

    let currency_gb = get_default_currency(&pool, Some("GB")).await.unwrap();
    assert_eq!(currency_gb, "GBP");

    let currency_unknown = get_default_currency(&pool, Some("ZZ")).await.unwrap();
    assert_eq!(currency_unknown, "USD");

    let currency_none = get_default_currency(&pool, None).await.unwrap();
    assert_eq!(currency_none, "USD");

    let name = FamilyName::try_new("The British Millers", "TEST").unwrap();
    let gbp_id = sqlx::query_scalar::<_, i64>("SELECT id FROM currencies WHERE code = 'GBP';")
        .fetch_one(&pool)
        .await
        .unwrap();
    update_family(&pool, Some(name), gbp_id).await.unwrap();

    let updated_currency = get_default_currency(&pool, None).await.unwrap();
    assert_eq!(updated_currency, "GBP");
}

#[tokio::test]
async fn test_family_schema_unique_constraint_violations() {
    let pool = setup_test_db().await;

    // 1. Family name unique constraint
    sqlx::query(
        "INSERT INTO families (family_name, currency_id, created_at, updated_at) VALUES ('My Family', 1, 0, 0);",
    )
    .execute(&pool)
    .await
    .unwrap();

    let dup_family_err = sqlx::query(
        "INSERT INTO families (family_name, currency_id, created_at, updated_at) VALUES ('My Family', 1, 0, 0);",
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

    // Insert Sarah Miller first
    sqlx::query(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'Sarah Miller', 0, 0);",
    )
    .bind(family_id)
    .execute(&pool)
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
        "INSERT INTO families (family_name, currency_id, created_at, updated_at) VALUES ('The Jones Family', 2, 0, 0) RETURNING id;",
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

    // 4. Account name unique for a member
    let (sarah_id,): (i64,) =
        sqlx::query_as("SELECT id FROM members WHERE member_name = 'Sarah Miller';")
            .fetch_one(&pool)
            .await
            .unwrap();

    sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', 1, 'Chase', '1234', 'Total Checking', 1000, 0, 0);
        "#,
    )
    .bind(family_id)
    .bind(sarah_id)
    .execute(&pool)
    .await
    .unwrap();

    let dup_account_err = sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', 1, 'Chase', '9999', 'Total Checking', 1000, 0, 0);
        "#,
    )
    .bind(family_id)
    .bind(sarah_id)
    .execute(&pool)
    .await
    .expect_err("duplicate account name for same member must fail");
    assert!(crate::core::is_unique_violation(&dup_account_err));

    // 5. Account with same name for a DIFFERENT member must succeed
    let david_id: i64 = sqlx::query_scalar(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (?, 'David Miller', 0, 0) RETURNING id;",
    )
    .bind(family_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let diff_member_acc = sqlx::query(
        r#"
        INSERT INTO accounts (
            family_id, owner_member_id, type, currency_id, bank_name, last4,
            account_name, available_balance_cents, created_at, updated_at
        ) VALUES (?, ?, 'bank_account', 1, 'Chase', '8888', 'Total Checking', 2000, 0, 0);
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

    // 6. Currency region uniqueness
    let dup_region_err = sqlx::query(
        "INSERT INTO currencies (code, name, symbol, scale, sort_order, region, created_at, updated_at) VALUES ('NEW', 'New', '$', 2, 99, 'US', 0, 0);",
    )
    .execute(&pool)
    .await
    .expect_err("duplicate region in currencies table must fail");
    assert!(crate::core::is_unique_violation(&dup_region_err));
}
