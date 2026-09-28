# Backend Encapsulation & Visibility Standards

Standards for module boundaries, struct field encapsulation, accessor conventions, and visibility rules across the Rust Axum backend.

---

## 1. Core Architecture

The backend architecture enforces strict subsystem isolation, data encapsulation, and compiler-verified cleanliness:

1. **Folder Facade Pattern**: Every subsystem folder (`core/`, `features/<feature>/`) presents a unified public API through its root `mod.rs`. Internal files (`db.rs`, `db/`, `models.rs`, `routes.rs`, `security.rs`) are internal implementation details declared private to the folder.
2. **Struct Property Encapsulation**: All struct properties are private. Direct field access (`pub` or `pub(crate)` on fields) is forbidden. Callers interact with structs through reference getters (`item.name() -> &str`), move consumers (`payload.into_parts(...)`), and explicit constructors.
3. **Visibility Hierarchy**:
   - `pub`: Errors (`AppError`, `CategoryError`, `AuthError`), error inspector methods (`action()`, `code()`), and essential framework runtime constructs (`DbPool`, `AppState`, `ApiResponse`, `Cli`).
   - `pub(crate)`: Feature domain models, DTOs, and functions shared across features through folder facades.
   - Private / `pub(super)`: Internal submodules and helpers inside a folder.
4. **Zero Dead Code**: `#![deny(dead_code)]` is enforced crate-wide. Unused code must be deleted immediately; `#[allow(dead_code)]` annotations are prohibited.
5. **Grouped Import Hierarchy**: All `use` statements reside at the top of the file before any declarations (unless strictly prevented by language constraints). Imports are grouped into 4 distinct tiers separated by a single blank line: (1) `std::*`, (2) third-party external crates, (3) `crate::*` internal modules, (4) `super::*` local subsystem items.

---

## 2. Invariants

1. **Folder Boundary Seam**: Once a folder boundary is crossed, external callers must access items strictly through the root folder name (`core::create_db_object`, `categories::init_schema`). Calling into internal submodules (`core::db::create_db_object`, `categories::db::init_schema`) is strictly prohibited. Submodules in `mod.rs` are declared without `pub(crate)` (e.g. `mod db; mod models;`).
2. **Private Struct Fields**: Struct fields must never carry `pub` or `pub(crate)` visibility. All fields remain private to their defining module.
3. **Reference Getters for Read Access**: Expose field data through accessor methods returning borrowed references or copyable primitives (`item.name() -> &str`, `item.sort_order() -> i64`, `state.db() -> &DbPool`).
4. **Move Consumers for Owned Data**: Consume owned payloads or transfer ownership using `into_parts(...)` tuples or dedicated `into_<field>()` methods.
5. **Explicit Constructors**: Construct instances via `new(...)` constructors or builder methods (`with_subcategories(...)`).
6. **Public Error Visibility**: Errors and their query methods (`action() -> &'static str`, `code() -> &'static str`) are `pub`. Domain errors represent the public contract of a subsystem failure.
7. **No Dead Code Annotations**: Do not suppress compiler warnings with `#[allow(dead_code)]`. If a model, variant, or function has no active consumers, remove it from the codebase.
8. **Top-of-File Grouped Imports**: Place all `use` declarations at the top of the file. Group imports into four sections separated by a single blank line: `std` first, then third-party dependencies, then `crate::`, and finally `super::`. Never scatter inline `use` declarations inside function bodies unless strictly required by conditional compilation.

---

## 3. Canonical Patterns

### Canonical Struct Encapsulation: `features/categories/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTypeResponse {
    id: String,
    name: String,
    sort_order: i64,
    categories: Vec<CategoryResponse>,
}

impl CategoryTypeResponse {
    pub(crate) fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        sort_order: i64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sort_order,
            categories: Vec::new(),
        }
    }

    pub(crate) fn with_categories(mut self, categories: Vec<CategoryResponse>) -> Self {
        self.categories = categories;
        self
    }

    // Reference getters
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn sort_order(&self) -> i64 {
        self.sort_order
    }

    pub(crate) fn categories(&self) -> &[CategoryResponse] {
        &self.categories
    }

    // Move consumer for owned destructuring
    pub(crate) fn into_parts(self) -> (String, String, i64, Vec<CategoryResponse>) {
        (self.id, self.name, self.sort_order, self.categories)
    }
}
```

### Canonical Request Payload with Move Consumer

```rust
#[derive(Debug, Deserialize)]
pub(crate) struct CreateCategoryTypePayload {
    name: String,
}

impl CreateCategoryTypePayload {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn into_name(self) -> String {
        self.name
    }
}
```

### Canonical Core State Encapsulation: `core/state.rs`

```rust
use crate::core::DbPool;

#[derive(Clone)]
pub struct AppState {
    db: DbPool,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DbPool {
        &self.db
    }
}
```

### Canonical Folder Facade: `core/mod.rs`

```rust
// Internal submodules declared private to the folder
#[cfg(feature = "cli")]
mod cli;
mod db;
mod error;
mod response;
mod state;

// Re-export the public boundary contract at root level
#[cfg(feature = "cli")]
pub use cli::Cli;
pub(crate) use db::{create_db_object, db_err, DbResultExt};
pub use db::{init_db, DbPool};
pub use error::AppError;
pub(crate) use response::{ApiResponse, Code, ErrorPayload, Status};
pub use state::AppState;
```

### Canonical Symmetrical Feature Facade: `features/<feature>/mod.rs`

Every domain feature implements the identical facade signatures:

```rust
use axum::Router;
use crate::core::AppState;

mod db;
mod error;
mod models;
mod routes;

// 1. Database schema initialization with uniform (pool: &DbPool) signature
pub(crate) use db::init_schema;

// 2. Feature error enum is the ONLY public export from the feature
pub use error::CategoryError;

// 3. Symmetrical router mounting
pub(crate) fn router() -> Router<AppState> {
    routes::router()
}
```

### Canonical Crate Root Exposure: `lib.rs`

Lowest visibility first: root modules remain strictly private; only required structs and error enums are exported:

```rust
#![deny(dead_code)]

mod core;
mod features;

#[cfg(feature = "cli")]
pub use core::Cli;
pub use core::{init_db, AppError, AppState, DbPool};
pub use features::{init_features, router, AuthError, CategoryError};
```

### Canonical Cross-Folder Imports

```rust
// In features/auth/db.rs:
use crate::core::{create_db_object, DbPool, DbResultExt};

// In features/categories/routes.rs:
use crate::core::{ApiResponse, AppState};

// In main.rs:
use cosave::{init_db, init_features, router, AppState, Cli};
```

### Canonical Import Hierarchy & Ordering

Imports must be placed at the top of the file, divided into four tiers separated by single blank lines:

```rust
// 1. Standard library
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// 2. External third-party crates
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// 3. First-party crate items (accessed strictly via folder facades)
use crate::core::{ApiResponse, AppState, DbPool, DbResultExt};
use crate::features::categories;

// 4. Local parent/sibling items within same subsystem folder
use super::error::AuthError;
use super::models::UserDto;
```

---

## 4. Contrast: Anti-Patterns vs Canonical Patterns

### Anti-Pattern ❌ (Public Properties, Leaked Submodules, Dead Code, Mixed Imports)

```rust
// AVOID: Public fields leak internal layout and break encapsulation
pub struct AppState {
    pub db: DbPool,
}

// AVOID: Struct fields marked pub(crate)
pub(crate) struct CategoryTypeResponse {
    pub(crate) id: String,
    pub(crate) name: String,
}

// AVOID: Crossing folder boundaries into internal submodules
use crate::core::db::create_db_object;
use crate::features::categories::db::init_schema;

// AVOID: Preserving unused structs with allow annotations
#[allow(dead_code)]
pub(crate) struct UnusedRecord {
    id: String,
}

// AVOID: Jumbled, unordered imports and inline use inside function bodies
use super::models::UserDto;
use std::time::SystemTime;
use crate::core::AppState;
use axum::Json;

fn handler() {
    use crate::core::response::ApiResponse; // AVOID inline imports
}
```

### Canonical Pattern ✅ (Encapsulated Fields, Folder Facades, Clean Code, Tiered Imports)

```rust
// PREFER: Top-of-file 4-tier separated import groups
use std::time::SystemTime;

use axum::Json;

use crate::core::AppState;

use super::models::UserDto;

// PREFER: Private fields with reference getter
pub struct AppState {
    db: DbPool,
}

impl AppState {
    pub fn db(&self) -> &DbPool {
        &self.db
    }
}

// PREFER: Private fields with getters and move consumers
pub(crate) struct CategoryTypeResponse {
    id: String,
    name: String,
}

impl CategoryTypeResponse {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn into_parts(self) -> (String, String) {
        (self.id, self.name)
    }
}

// PREFER: Importing strictly through the root folder facade
use crate::core::create_db_object;
use crate::features::categories::init_schema;

// PREFER: Delete unused types immediately under #![deny(dead_code)]
```
