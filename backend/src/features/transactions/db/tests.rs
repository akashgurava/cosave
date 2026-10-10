//! Tier 2 direct database, transaction boundary, and rollback tests for transactions.

use crate::core::{init_db, now_epoch_secs, AppConfig, DbPool};
use crate::features::categories::init_category_schema;
use crate::features::family::init_family_schema;

use super::super::models::{CreateTransactionRequest, UpdateTransactionRequest};
use super::schema::init_transaction_schema;
use super::transactions::{
    create_manual_transaction, delete_transaction, get_transaction, update_transaction,
};

async fn setup_test_db() -> (DbPool, i64, i64, i64, i64) {
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

    let category_id: i64 = sqlx::query_scalar(
        "INSERT INTO categories (category_name, type_id, sort_order, created_at, updated_at) VALUES ('Groceries', ?, 1, ?, ?) RETURNING id;",
    )
    .bind(type_id)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .unwrap();

    (pool, family_id, account_id, type_id, category_id)
}

#[tokio::test]
async fn test_create_manual_transaction_and_view_projection() {
    let (pool, family_id, account_id, type_id, category_id) = setup_test_db().await;

    let payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Whole Foods Market",
        "payee": "Whole Foods",
        "amount": -8420,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    }))
    .unwrap();

    let tx_dto = create_manual_transaction(&pool, family_id, &payload)
        .await
        .expect("Create transaction should succeed");

    assert_eq!(tx_dto.source(), "manual");
    assert_eq!(tx_dto.description(), Some("Whole Foods Market"));
    assert_eq!(tx_dto.payee(), Some("Whole Foods"));
    assert_eq!(tx_dto.amount(), -8420);
    assert_eq!(tx_dto.date(), "2026-10-05");
    assert_eq!(tx_dto.status(), "cleared");

    // Verify 1:1 records in manual_transactions
    let manual_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions WHERE family_id = 1;")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(manual_count, 1);

    // Verify projected through transactions view
    let view_row =
        sqlx::query("SELECT id, source, description, amount FROM transactions WHERE id = ?;")
            .bind(tx_dto.id())
            .fetch_one(&pool)
            .await
            .unwrap();

    let source: String = sqlx::Row::get(&view_row, "source");
    let desc: Option<String> = sqlx::Row::get(&view_row, "description");
    let amt: i64 = sqlx::Row::get(&view_row, "amount");

    assert_eq!(source, "manual");
    assert_eq!(desc.as_deref(), Some("Whole Foods Market"));
    assert_eq!(amt, -8420);
}

#[tokio::test]
async fn test_create_manual_transaction_error_on_missing_account() {
    let (pool, family_id, _account_id, type_id, category_id) = setup_test_db().await;

    let payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Orphan Attempt",
        "amount": -5000,
        "typeId": type_id,
        "accountId": 9999, // Non-existent account
        "categoryId": category_id,
        "status": "cleared"
    }))
    .unwrap();

    let result = create_manual_transaction(&pool, family_id, &payload).await;
    assert!(result.is_err());

    // Verify zero rows in manual_transactions
    let manual_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions;")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(manual_count, 0);
}

#[tokio::test]
async fn test_update_manual_transaction_full_replacement() {
    let (pool, family_id, account_id, type_id, category_id) = setup_test_db().await;

    let create_payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Original Purchase",
        "payee": "Store A",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    }))
    .unwrap();

    let created = create_manual_transaction(&pool, family_id, &create_payload)
        .await
        .unwrap();

    let update_payload: UpdateTransactionRequest = serde_json::from_value(serde_json::json!({
        "source": "manual",
        "date": "2026-10-06",
        "description": "Updated Purchase",
        "payee": "Store B",
        "amount": -2000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "pending"
    }))
    .unwrap();

    let updated = update_transaction(&pool, family_id, created.id(), &update_payload)
        .await
        .expect("Update should succeed");

    assert_eq!(updated.source(), "manual");
    assert_eq!(updated.description(), Some("Updated Purchase"));
    assert_eq!(updated.payee(), Some("Store B"));
    assert_eq!(updated.amount(), -2000);
    assert_eq!(updated.date(), "2026-10-06");
    assert_eq!(updated.status(), "pending");

    // Attempting update with non-manual source returns UnsupportedOperation error
    let bad_update_payload: UpdateTransactionRequest = serde_json::from_value(serde_json::json!({
        "source": "import",
        "date": "2026-10-06",
        "description": "Unsupported Update",
        "amount": -2000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id
    }))
    .unwrap();

    let err = update_transaction(&pool, family_id, created.id(), &bad_update_payload)
        .await
        .unwrap_err();
    assert_eq!(err.code(), "UNSUPPORTED_OPERATION");
}

#[tokio::test]
async fn test_delete_transaction_by_source() {
    let (pool, family_id, account_id, type_id, category_id) = setup_test_db().await;

    // 1. Test manual transaction deletion
    let create_payload: CreateTransactionRequest = serde_json::from_value(serde_json::json!({
        "date": "2026-10-05",
        "description": "Delete Me Manual",
        "amount": -1000,
        "typeId": type_id,
        "accountId": account_id,
        "categoryId": category_id,
        "status": "cleared"
    }))
    .unwrap();

    let manual_tx = create_manual_transaction(&pool, family_id, &create_payload)
        .await
        .unwrap();

    delete_transaction(&pool, family_id, manual_tx.id(), "manual")
        .await
        .expect("Delete manual transaction should succeed");

    let fetched = get_transaction(&pool, family_id, manual_tx.id())
        .await
        .unwrap();
    assert!(fetched.is_none());

    let manual_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM manual_transactions WHERE id = ?;")
            .bind(manual_tx.id())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(manual_count, 0);

    // 2. Test imported transaction deletion
    let now = now_epoch_secs();
    let import_tx_id = uuid::Uuid::new_v4().to_string();

    // Insert prerequisite staging transaction
    sqlx::query(
        "INSERT INTO statement_imports (id, family_id, account_id, filename, file_hash, file_format, created_at) VALUES (1, 1, ?, 'test.csv', 'hash123', 'csv', ?);",
    )
    .bind(account_id)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO raw_statement_rows (id, import_id, row_index, raw_payload, created_at) VALUES (1, 1, 0, '{}', ?);",
    )
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO staging_transactions (id, raw_row_id, date, amount, description, created_at) VALUES (?, 1, ?, -500, 'Imported Staging', ?);",
    )
    .bind(&import_tx_id)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO imported_transactions (
            id, family_id, account_id, type_id, category_id, subcategory_id,
            amount, date, description, payee, notes, status, created_at, updated_at
        )
        VALUES (?, 1, ?, ?, ?, NULL, -500, ?, 'Imported Entry', NULL, NULL, 'cleared', ?, ?);
        "#,
    )
    .bind(&import_tx_id)
    .bind(account_id)
    .bind(type_id)
    .bind(category_id)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    let fetched_import = get_transaction(&pool, family_id, &import_tx_id)
        .await
        .unwrap();
    assert!(fetched_import.is_some());
    assert_eq!(fetched_import.unwrap().source(), "import");

    delete_transaction(&pool, family_id, &import_tx_id, "import")
        .await
        .expect("Delete imported transaction should succeed");

    let fetched_after_del = get_transaction(&pool, family_id, &import_tx_id)
        .await
        .unwrap();
    assert!(fetched_after_del.is_none());

    let imported_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM imported_transactions WHERE id = ?;")
            .bind(&import_tx_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(imported_count, 0);
}
