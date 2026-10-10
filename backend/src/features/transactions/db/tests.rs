//! Tier 2 direct database, transaction boundary, and rollback tests for transactions.

use crate::core::{init_db, now_epoch_secs, AppConfig, DbPool};
use crate::features::categories::init_category_schema;
use crate::features::family::init_family_schema;

use super::super::models::{CreateTransactionRequest, UpdateTransactionRequest};
use super::schema::init_transaction_schema;
use super::transactions::{
    create_manual_transaction, delete_transaction, get_transaction, update_transaction,
};

async fn setup_test_db() -> (DbPool, i64, i64, i64) {
    let pool = init_db(AppConfig::IN_MEMORY_DATABASE_URL)
        .await
        .expect("Failed to create in-memory test db");

    let mut tx = pool.begin().await.unwrap();
    init_category_schema(&mut tx).await.unwrap();
    init_family_schema(&mut tx).await.unwrap();
    init_transaction_schema(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let now = now_epoch_secs();

    // 1. Setup currency, family, member, and account
    let family_id: i64 = 1;
    sqlx::query(
        "INSERT INTO currencies (id, code, name, symbol, scale, sort_order, region, created_at, updated_at) VALUES (1, 'USD', 'US Dollar', '$', 2, 1, 'US', ?, ?);",
    )
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO families (id, family_name, currency_id, created_at, updated_at) VALUES (1, 'Test Family', 1, ?, ?);",
    )
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    let member_id: i64 = sqlx::query_scalar(
        "INSERT INTO members (family_id, member_name, created_at, updated_at) VALUES (1, 'Alice', ?, ?) RETURNING id;",
    )
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    let account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (family_id, owner_member_id, type, currency_id, bank_name, account_name, last4, available_balance, created_at, updated_at) VALUES (1, ?, 'bank_account', 1, 'Chase', 'Checking', '1234', 500000, ?, ?) RETURNING id;",
    )
    .bind(member_id)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Setup color and transaction type
    let color_id: i64 = sqlx::query_scalar(
        "INSERT INTO colors (name, hex, sort_order, created_at, updated_at) VALUES ('Rose', '#f43f5e', 1, ?, ?) RETURNING id;",
    )
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    let type_id: i64 = sqlx::query_scalar(
        "INSERT INTO transaction_types (type_name, color_id, sort_order, created_at, updated_at) VALUES ('Expense', ?, 1, ?, ?) RETURNING id;",
    )
    .bind(color_id)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    (pool, family_id, account_id, type_id)
}

#[tokio::test]
async fn test_create_manual_transaction_and_lineage() {
    let (pool, family_id, account_id, type_id) = setup_test_db().await;

    let payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Whole Foods Market",
        "payee": "Whole Foods",
        "amount": -8420,
        "typeId": type_id,
        "accountId": account_id,
        "status": "cleared"
    }))
    .unwrap();

    let tx_dto = create_manual_transaction(&pool, family_id, &payload)
        .await
        .expect("Create transaction should succeed");

    assert_eq!(tx_dto.description(), "Whole Foods Market");
    assert_eq!(tx_dto.payee(), Some("Whole Foods"));
    assert_eq!(tx_dto.amount(), -8420);
    assert_eq!(tx_dto.date(), "2026-10-05");
    assert_eq!(tx_dto.status(), "cleared");

    // Verify 1:1 records in manual_transactions and transaction_sources
    let manual_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions WHERE family_id = 1;")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(manual_count, 1);

    let source_row = sqlx::query(
        "SELECT id, source_type, manual_id, staging_id FROM transaction_sources WHERE id = ?;",
    )
    .bind(tx_dto.id())
    .fetch_one(&pool)
    .await
    .unwrap();

    let source_type: String = sqlx::Row::get(&source_row, "source_type");
    let manual_id: Option<i64> = sqlx::Row::get(&source_row, "manual_id");
    let staging_id: Option<i64> = sqlx::Row::get(&source_row, "staging_id");

    assert_eq!(source_type, "manual");
    assert!(manual_id.is_some());
    assert!(staging_id.is_none());
}

#[tokio::test]
async fn test_create_manual_transaction_rollback_on_missing_account() {
    let (pool, family_id, _account_id, type_id) = setup_test_db().await;

    let payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Orphan Attempt",
        "amount": -5000,
        "typeId": type_id,
        "accountId": 9999, // Non-existent account
        "status": "cleared"
    }))
    .unwrap();

    let result = create_manual_transaction(&pool, family_id, &payload).await;
    assert!(result.is_err());

    // Verify atomic rollback left zero rows
    let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions;")
        .fetch_one(&pool)
        .await
        .unwrap();
    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transaction_sources;")
        .fetch_one(&pool)
        .await
        .unwrap();
    let manual_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions;")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(tx_count, 0);
    assert_eq!(source_count, 0);
    assert_eq!(manual_count, 0);
}

#[tokio::test]
async fn test_update_transaction_flips_import_source_to_manual() {
    let (pool, family_id, account_id, type_id) = setup_test_db().await;
    let now = now_epoch_secs();

    // Seed imported transaction directly
    let source_id: i64 = sqlx::query_scalar(
        "INSERT INTO transaction_sources (family_id, source_type, staging_id, manual_id, created_at, updated_at) VALUES (1, 'import', NULL, NULL, ?, ?) RETURNING id;",
    )
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO transactions (id, family_id, account_id, member_id, type_id, category_id, subcategory_id, amount, date, description, payee, notes, status, created_at, updated_at) VALUES (?, 1, ?, 1, ?, NULL, NULL, -1500, ?, 'RAW IMPORT DESC', NULL, NULL, 'cleared', ?, ?);",
    )
    .bind(source_id)
    .bind(account_id)
    .bind(type_id)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    // User updates the imported transaction manually
    let update_payload: UpdateTransactionRequest = serde_json::from_value(serde_json::json!({
        "payee": "Starbucks Coffee",
        "description": "Coffee at Market St"
    }))
    .unwrap();

    let updated = update_transaction(&pool, family_id, source_id, &update_payload)
        .await
        .expect("Update should succeed");

    assert_eq!(updated.description(), "Coffee at Market St");
    assert_eq!(updated.payee(), Some("Starbucks Coffee"));

    // Verify source_type flipped to 'manual' and a new manual_transaction row was created
    let source_row =
        sqlx::query("SELECT source_type, manual_id FROM transaction_sources WHERE id = ?;")
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let source_type: String = sqlx::Row::get(&source_row, "source_type");
    let manual_id: Option<i64> = sqlx::Row::get(&source_row, "manual_id");

    assert_eq!(source_type, "manual");
    assert!(manual_id.is_some());

    let manual_row =
        sqlx::query("SELECT description, payee FROM manual_transactions WHERE id = ?;")
            .bind(manual_id.unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let m_desc: String = sqlx::Row::get(&manual_row, "description");
    let m_payee: Option<String> = sqlx::Row::get(&manual_row, "payee");

    assert_eq!(m_desc, "Coffee at Market St");
    assert_eq!(m_payee.as_deref(), Some("Starbucks Coffee"));
}

#[tokio::test]
async fn test_delete_transaction_cascades_and_cleans_up() {
    let (pool, family_id, account_id, type_id) = setup_test_db().await;

    let payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Delete Me",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id,
        "status": "cleared"
    }))
    .unwrap();

    let created = create_manual_transaction(&pool, family_id, &payload)
        .await
        .unwrap();

    delete_transaction(&pool, family_id, created.id())
        .await
        .expect("Delete should succeed");

    let fetched = get_transaction(&pool, family_id, created.id())
        .await
        .unwrap();
    assert!(fetched.is_none());

    let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions WHERE id = ?;")
        .bind(created.id())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tx_count, 0);

    let manual_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions WHERE family_id = 1;")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(manual_count, 0);
}
