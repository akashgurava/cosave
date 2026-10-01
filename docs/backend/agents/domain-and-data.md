# Backend Domain & Data Standards

Authoritative standards for SQLite database schemas, transaction boundaries, identifier conventions, timestamp management, and domain modeling via Value Objects ("Parse, Don't Validate").

---

## 1. Core Architecture

The backend domain and persistence layers guarantee transactional integrity, auditability, and invariant preservation from memory to disk:

1. **Idempotent DDL & Explicit Referential Actions**: All tables, indexes, and views are declared via `crate::core::create_db_object(action, table, pool, sql)`. Foreign keys must explicitly specify referential cascades (`ON DELETE CASCADE`, `ON DELETE RESTRICT`). Implicit SQLite defaults are forbidden.
2. **Atomic Transaction Boundaries**: Any business workflow executing multiple write operations (INSERT, UPDATE, DELETE) or combining state validation with a subsequent write must execute inside an explicit database transaction (`pool.begin().await`). Sub-operations participating in a transaction accept `&mut sqlx::Transaction<'_, sqlx::Sqlite>`. Non-database computations (password hashing, disk I/O, external network calls) are strictly forbidden inside active transactions.
3. **Entity Identifiers (User String vs. Integer Primary Keys)**: Only user entities use typed, prefixed string identifiers (`usr_<id>`). All other persistent domain entities (`colors`, `transaction_types`, `categories`, `subcategories`, `families`, `members`, `accounts`, etc.) use 64-bit auto-incrementing integer primary keys (`INTEGER PRIMARY KEY AUTOINCREMENT` in SQLite, `i64` in Rust). System metadata and declarative seed execution flags are stored in the `app_meta` table (`key TEXT PRIMARY KEY`, `value TEXT`, `updated_at INTEGER`).
4. **Timestamp SSOT**: Database timestamps are stored strictly as `INTEGER NOT NULL` representing UTC epoch seconds. Timestamps are sourced exclusively from `crate::core::time::now_epoch_secs()`. Feature-level ad-hoc timestamp generators, floating-point timestamps, and ISO date strings in SQLite columns are forbidden.
5. **Domain Invariants & Value Objects ("Parse, Don't Validate")**: Eliminate primitive obsession by wrapping domain primitives in dedicated Rust newtypes (e.g. `CategoryName`, `HexColor`, `AmountCents`). Value Objects can only be instantiated through fallible constructors (`try_new(raw, action) -> Result<Self, FeatureError>`). Invalid domain states are unrepresentable in memory. Financial values are strictly represented as integer minor units (`AmountCents(i64)`); floating-point types (`f32`, `f64`) for money are prohibited.

---

## 2. Invariants

1. **Idempotent Schema Definition**: Every DDL migration must use `CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`, or `CREATE VIEW IF NOT EXISTS`.
2. **Explicit Foreign Key Constraint**: Every foreign key column must include an explicit referential constraint:
   - `REFERENCES parent_table(id) ON DELETE CASCADE` (for child resources whose lifecycle depends on the parent)
   - `REFERENCES parent_table(id) ON DELETE RESTRICT` (to prevent orphaned dependencies)
   - Implicit foreign key definitions without `ON DELETE` are strictly forbidden.
3. **Index Naming Standard**: Indexes must follow the canonical naming convention: `idx_<table_name>_<column1>[_<column2>]`.
4. **Dedicated Transaction Step Tokens**: Transaction lifecycle steps must each carry distinct compile-time action tokens:
   - `FEATURE.WORKFLOW.TX_BEGIN` on `pool.begin().await`
   - `FEATURE.WORKFLOW.STEP.QUERY` on each query executed within the transaction
   - `FEATURE.WORKFLOW.TX_COMMIT` on `tx.commit().await`
5. **No Long-Running Transaction Blocks**: CPU-intensive operations (such as password hashing via Argon2id) or external network calls must be executed before opening or after committing the transaction. Never hold a SQLite write lock while computing non-database work.
6. **Time Representation**: All SQLite timestamp columns must be named `created_at`, `updated_at`, or `<event>_at`, typed as `INTEGER NOT NULL`, and populated with UTC epoch seconds (`i64`).
7. **Value Object Encapsulation**: Value Object internal fields are private. Access is provided strictly via `.as_str()`, `.get()`, or `.into_inner()`. Callers cannot mutate or bypass Value Object validation.
8. **Integer Currency Rule**: All monetary values are integer cents (`i64`). Never perform currency calculations using floating-point types (`f32` or `f64`).

---

## 3. Canonical Patterns

### Canonical Value Object: `CategoryName`

```rust
use super::error::CategoryError;

/// Validated category name value object enforcing non-empty and length constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CategoryName(String);

impl CategoryName {
    pub(crate) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyName { action });
        }
        if trimmed.len() > 64 {
            return Err(CategoryError::NameTooLong {
                action,
                name: trimmed,
                max: 64,
            });
        }
        Ok(Self(trimmed))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}
```

### Canonical Monetary Value Object: `AmountCents`

```rust
use super::error::TransactionError;

/// Strongly-typed monetary value represented in integer minor units (cents).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct AmountCents(i64);

impl AmountCents {
    pub(crate) fn new(cents: i64) -> Self {
        Self(cents)
    }

    pub(crate) fn from_major_units(
        units: i64,
        fractional_cents: i64,
        action: &'static str,
    ) -> Result<Self, TransactionError> {
        let total = units
            .checked_mul(100)
            .and_then(|u| u.checked_add(fractional_cents))
            .ok_or(TransactionError::AmountOverflow { action })?;
        Ok(Self(total))
    }

    pub(crate) fn cents(&self) -> i64 {
        self.0
    }
}
```

### Canonical Atomic Transaction with Granular Action Tokens

```rust
use sqlx::{Sqlite, Transaction};
use crate::core::{AppError, DbPool, DbResultExt};
use super::models::{CategoryItem, CategoryName};

pub(crate) async fn reorder_categories(
    pool: &DbPool,
    category_ids: &[String],
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.REORDER.TX_BEGIN")?;

    for (index, id) in category_ids.iter().enumerate() {
        update_category_sort_order(&mut tx, id, index as i64).await?;
    }

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.REORDER.TX_COMMIT")?;

    Ok(())
}

async fn update_category_sort_order(
    tx: &mut Transaction<'_, Sqlite>,
    category_id: &str,
    sort_order: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE categories SET sort_order = ?, updated_at = ? WHERE id = ?"
    )
    .bind(sort_order)
    .bind(crate::core::time::now_epoch_secs())
    .bind(category_id)
    .execute(&mut **tx)
    .await
    .db_context("CONFIG.CATEGORIES.REORDER.STEP.UPDATE_SORT_ORDER")?;

    Ok(())
}
```

### Canonical DDL Initialization

```rust
use crate::core::{create_db_object, AppError, DbPool};

pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), AppError> {
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE",
        "categories",
        pool,
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY NOT NULL,
            type_id TEXT NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX_TYPE_ID",
        "categories",
        pool,
        "CREATE INDEX IF NOT EXISTS idx_categories_type_id ON categories(type_id);",
    )
    .await?;

    Ok(())
}
```

---

## 4. Anti-Patterns

### Anti-Pattern 1: Primitive Obsession vs Value Objects

```rust
// ❌ Anti-Pattern: Unvalidated raw strings passed throughout handlers and queries
pub(crate) async fn create_category(pool: &DbPool, name: String) -> Result<Category, AppError> {
    if name.trim().is_empty() {
        return Err(CategoryError::InvalidName { ... }.into());
    }
    // Database query receives bare String, validation logic scattered across call sites
}

// ✅ Canonical: Constructor parses and guarantees invariants once at boundary
let valid_name = CategoryName::try_new(payload.name, "CONFIG.CATEGORIES.CREATE.PARSE_NAME")?;
let created = db::create_category(pool, valid_name).await?;
```

### Anti-Pattern 2: Implicit Foreign Keys & Ad-Hoc Timestamps

```rust
// ❌ Anti-Pattern: Missing referential action, ISO string timestamp, integer auto-increment for domain entity
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type_id TEXT REFERENCES transaction_types(id), -- missing ON DELETE action!
    created_at TEXT NOT NULL -- string timestamps cause sorting and query drift!
);

// ✅ Canonical: Prefixed string ID, explicit ON DELETE, integer epoch seconds
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    type_id TEXT NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Anti-Pattern 3: Side Effects Inside Database Transactions

```rust
// ❌ Anti-Pattern: Password hashing performed inside an open SQLite write transaction
let mut tx = pool.begin().await.db_context("AUTH.REGISTER.TX_BEGIN")?;
let password_hash = hash_password(&password)?; // Argon2id takes 100-300ms, locking the database!
sqlx::query("INSERT INTO users ...").execute(&mut *tx).await...;
tx.commit().await...;

// ✅ Canonical: Compute intensive work BEFORE acquiring transaction lock
let password_hash = hash_password(&password)?;
let mut tx = pool.begin().await.db_context("AUTH.REGISTER.TX_BEGIN")?;
sqlx::query("INSERT INTO users ...").execute(&mut *tx).await...;
tx.commit().await.db_context("AUTH.REGISTER.TX_COMMIT")?;
```

---

## 5. Checklist for Agents

Before completing any database or domain modeling task:

- [ ] All tables use `CREATE TABLE IF NOT EXISTS` and all indexes use `CREATE INDEX IF NOT EXISTS`.
- [ ] Every foreign key has an explicit `ON DELETE CASCADE`, `ON DELETE RESTRICT`, or `ON DELETE SET NULL` constraint.
- [ ] Indexes follow the naming convention `idx_<table_name>_<column1>[_<column2>]`.
- [ ] Multi-statement write workflows or check-then-write logic run within `pool.begin().await` transactions.
- [ ] Transaction lifecycle steps have unique compile-time action tokens (`TX_BEGIN`, step queries, `TX_COMMIT`).
- [ ] Heavy compute (password hashing, encryption) and I/O (files, HTTP requests) are executed outside transaction blocks.
- [ ] Domain entity primary keys use prefixed collision-free string IDs (`usr_`, `cat_`, etc.).
- [ ] Timestamps are stored as `INTEGER NOT NULL` (epoch seconds) using `crate::core::time::now_epoch_secs()`.
- [ ] Core domain fields with validation rules are implemented as private Rust newtype Value Objects.
- [ ] Monetary quantities are stored and calculated strictly as integer cents (`i64`), never floats (`f32`/`f64`).
