# Backend Domain & Data Standards

Authoritative standards for SQLite database schemas, transaction boundaries, identifier conventions, timestamp management, and domain modeling via Value Objects ("Parse, Don't Validate").

---

## 1. Core Architecture

The backend domain and persistence layers guarantee transactional integrity, auditability, and invariant preservation from memory to disk:

1. **Idempotent DDL & Explicit Referential Actions**: All tables, indexes, and views are declared via `crate::core::create_db_object(action, table, tx, sql)` within active migration transactions. Foreign keys must explicitly specify referential cascades (`ON DELETE CASCADE`, `ON DELETE RESTRICT`). Implicit SQLite defaults are forbidden.
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
8. **Integer Currency Rule**: All monetary values are integer minor units (`i64`), scale-aware according to the currency's ISO 4217 minor unit exponent (e.g. scale 2 for USD/EUR/INR, scale 0 for JPY, scale 3 for KWD). Never perform currency calculations using floating-point types (`f32` or `f64`).
9. **Single-Shot Atomic Database Writes**: Single-entity insertions with sequential display ordering must compute sort order within the SQL statement itself (`(SELECT COALESCE(MAX(sort_order), 0) + 1 FROM <table> [WHERE parent_id = ?])`) rather than opening multi-step transactions with separate `SELECT MAX` round trips.
10. **Database Constraint Failure Classification**: Relational integrity (`REFERENCES`) and uniqueness (`UNIQUE`) must be enforced at the SQLite engine level. Failures are caught via `crate::core::is_foreign_key_violation` and `crate::core::is_unique_violation` (inspecting `sqlx::error::ErrorKind`) and mapped to descriptive feature domain errors.
11. **Canonical Hierarchy Column Ordering**: Denormalized hierarchy views (`v_category_hierarchy`) standardize on the canonical 11-column sequence:
    `type_color_id, type_color, type_id, type_name, type_sort_order, category_id, category_name, category_sort_order, subcategory_id, subcategory_name, subcategory_sort_order`.
12. **Authoritative Database & Schema Documentation**: Database routines must document their execution model (single-shot atomic vs multi-statement transaction), `# Ingress`, `# Returns`, and `# Errors`. Schema initialization routines must document `# Database Objects Created` (tables, constraints, cascades, indexes, views) and `# Invariants`. Follow [`docs/backend/agents/documentation.md`](documentation.md).

---

## 3. Canonical Patterns

### Canonical Value Object: `CategoryName`
 
```rust
use super::error::CategoryError;

/// Validated category name Value Object ("Parse, Don't Validate").
///
/// Guarantees that empty or whitespace-only category names cannot be represented.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct CategoryName(String);

impl CategoryName {
    /// Trims the input and validates non-emptiness.
    ///
    /// # Errors
    /// Returns [`CategoryError::EmptyCategoryName`] if the trimmed string is empty.
    pub(super) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> {
        let trimmed = raw.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(CategoryError::EmptyCategoryName { action });
        }
        Ok(Self(trimmed))
    }

    /// Borrows the validated inner name string slice.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    /// Unwraps and consumes into the owned name [`String`].
    pub(super) fn into_inner(self) -> String {
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

pub(crate) async fn reset_defaults(pool: &DbPool) -> Result<(), AppError> {
    let now = crate::core::time::now_epoch_secs();
    let mut tx = pool
        .begin()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.BEGIN_TRANSACTION")?;

    tx.execute("DELETE FROM subcategories")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_SUBCATEGORIES")?;
    tx.execute("DELETE FROM categories")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_CATEGORIES")?;
    tx.execute("DELETE FROM transaction_types")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_TRANSACTION_TYPES")?;
    tx.execute("DELETE FROM colors")
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.DELETE_COLORS")?;

    seed_hierarchy_from_json(&mut tx, now).await?;

    tx.commit()
        .await
        .db_context("CONFIG.CATEGORIES.RESET_DEFAULTS.COMMIT_TRANSACTION")?;

    Ok(())
}
```

### Canonical Single-Shot Atomic Insertion: `create_category`

```rust
use crate::core::{
    db_err, is_foreign_key_violation, is_unique_violation, now_epoch_secs, AppError, DbPool,
};
use super::models::{CategoryItem, CategoryName, CreateCategoryRequest};
use super::error::CategoryError;

pub(crate) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> {
    let name = CategoryName::try_new(payload.name(), "CONFIG.CATEGORIES.CREATE_CATEGORY.EMPTY_NAME")?;
    let type_id = payload.type_id();
    let now = now_epoch_secs();
    let raw_name = name.into_inner();

    let res = sqlx::query(
        r#"
        INSERT INTO categories (type_id, category_name, sort_order, created_at, updated_at)
        VALUES (?, ?, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM categories WHERE type_id = ?), ?, ?)
        "#,
    )
    .bind(type_id)
    .bind(&raw_name)
    .bind(type_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await;

    match res {
        Ok(exec) => Ok(CategoryItem::new(exec.last_insert_rowid(), raw_name)),
        Err(err) => {
            if is_unique_violation(&err) {
                Err(CategoryError::CategoryAlreadyExists {
                    action: "CONFIG.CATEGORIES.CREATE_CATEGORY.ALREADY_EXISTS",
                    name: raw_name,
                }.into())
            } else if is_foreign_key_violation(&err) {
                Err(CategoryError::TypeNotFound {
                    action: "CONFIG.CATEGORIES.CREATE_CATEGORY.TYPE_NOT_FOUND",
                    id: type_id.to_string(),
                }.into())
            } else {
                Err(db_err("CONFIG.CATEGORIES.CREATE_CATEGORY.INSERT", err))
            }
        }
    }
}
```

### Canonical DDL Initialization

```rust
use sqlx::{Sqlite, Transaction};
use crate::core::{create_db_object, AppError};

pub(crate) async fn init_category_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), AppError> {
    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_TABLE",
        "categories",
        tx,
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            type_id INTEGER NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
            category_name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(type_id, category_name)
        );
        "#,
    )
    .await?;

    create_db_object(
        "CONFIG.CATEGORIES.INIT_SCHEMA.CATEGORIES_INDEX_TYPE_ID",
        "categories",
        tx,
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
// ❌ Anti-Pattern: Missing referential action, ISO string timestamp
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    type_id INTEGER REFERENCES transaction_types(id), -- missing ON DELETE action!
    category_name TEXT NOT NULL,
    created_at TEXT NOT NULL -- string timestamps cause sorting and query drift!
);

// ✅ Canonical: Explicit ON DELETE CASCADE, integer epoch seconds
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    type_id INTEGER NOT NULL REFERENCES transaction_types(id) ON DELETE CASCADE,
    category_name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(type_id, category_name)
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
- [ ] User entities use prefixed string IDs (`usr_...`), while persistent taxonomy and domain entities use 64-bit integer primary keys (`i64`, `INTEGER PRIMARY KEY AUTOINCREMENT`).
- [ ] Timestamps are stored as `INTEGER NOT NULL` (epoch seconds) using `crate::core::time::now_epoch_secs()`.
- [ ] Core domain fields with validation rules are implemented as private Rust newtype Value Objects.
- [ ] Monetary quantities are stored and calculated strictly as integer cents (`i64`), never floats (`f32`/`f64`).
