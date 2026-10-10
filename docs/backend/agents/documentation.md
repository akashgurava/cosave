# Backend Documentation & Rustdoc Standards

Authoritative engineering standards for module-level (`//!`) and item-level (`///`) Rust documentation, contract specifications, section hierarchy, and synchronization invariants across the CoSave backend.

---

## 1. Core Philosophy & Architectural Intent

In CoSave, documentation is not passive commentary or after-the-fact decoration. Rustdoc comments are executable, authoritative contract specifications:

1. **Dual-Tier Documentation Coverage**:
   - **Outer Module Comments (`//!`)**: Every Rust source file begins with a top-level module doc comment defining the module's architectural role, lifecycle, transaction boundaries, CQS invariants, and capability breakdown.
   - **Inner Item Comments (`///`)**: Every public, `pub(crate)`, `pub(super)`, and non-trivial private item (functions, structs, enums, traits, methods, type aliases, constants) carries a structured doc comment.
2. **Standardized Sectional Hierarchy**: Function and handler doc comments adhere to rigid Markdown section headings (`# Endpoint Contract`, `# Security & Access Control`, `# Ingress`, `# Returns` / `# Egress`, `# Errors` / `# Failure Contract`, `# Invariants`, `# Database Objects Created`).
3. **Living Documentation & Zero Stale Contracts**: Doc comments must stay in lockstep with implementation refactorings. If a route moves under a new prefix (e.g. `/config`), a multi-step transaction collapses into a single-shot atomic query, or an endpoint transitions to Command-Query Separation (CQS), doc comments must be updated in the same changeset. Stale doc comments are treated with the same severity as compiler type errors.
4. **Intra-Doc Cross Linking**: Use Rustdoc intra-doc links (`[`TypeName`]`, [`AppError::ShouldNotBeHappening`]) to maintain navigation coherence across subsystem boundaries.

---

## 2. Invariants & Rules

1. **Mandatory Module Header (`//!`)**:
   - The very first line of every `.rs` file must be a concise, active single-sentence summary of the module's role ending with a period.
   - The summary is followed by an empty `//!` line, a detailed architectural description paragraph, and bulleted highlights detailing key capabilities and guarantees.
2. **Mandatory Item Header (`///`)**:
   - Every struct, enum, trait, function, and method must begin with a concise summary sentence in third-person present tense ("Retrieves the...", "Creates a...", "Validates and trims...").
3. **Route Handler Documentation Contract**:
   - Must specify the canonical HTTP method and path (`Canonical route: GET /api/v1/config/hierarchy`), along with all supported aliases (`Aliases: GET /api/v1/config/hierarchies, GET /api/v1/config/categories/hierarchy`).
   - Must document authentication requirements, role authorization guards (`require_admin`), and tenant resource scoping (`family_id`).
   - Must include `# Ingress` detailing injected Axum extractors (`State`, `Path`, `Query`, `Json`, `CookieJar`).
   - Must include `# Returns` (or `# Egress`) detailing HTTP status codes, standard `ApiResponse<T>` envelopes, and Command-Query Separation guarantees (e.g., returning `ApiResponse<()>` without read amplification).
   - Must include `# Errors` (or `# Failure Contract`) mapping HTTP status codes (`400`, `401`, `403`, `404`, `409`, `500`) to exact domain error variants.
4. **Database Function Documentation Contract**:
   - Must clearly specify the database execution model: single-shot atomic statement with inline subqueries (e.g. `(SELECT COALESCE(MAX(sort_order), 0) + 1 FROM ...)`) vs multi-statement transaction (`Transaction<'_, Sqlite>`).
   - Must document rows-affected validation and SQLite engine-level constraint classification (`is_unique_violation`, `is_foreign_key_violation`).
   - Must include `# Ingress`, `# Returns`, and `# Errors` listing exact domain error variants and `AppError` cases.
5. **Schema & DDL Documentation Contract**:
   - Focus on high-signal architectural and domain information: entity relationships, domain rules, cascade behaviors (`ON DELETE CASCADE|RESTRICT`), uniqueness scopes, and performance indexing.
   - Do NOT copy-paste raw SQL column definitions, types, or constraints into doc comments—that duplicates the literal DDL residing directly in the function body below.
   - Must include `# Domain Rules & Referential Integrity` detailing business rules embedded in schema (cascades, deletion restrictions, uniqueness scopes).
   - Must include `# Execution & Idempotency` specifying transaction lifecycle and idempotent DDL (`CREATE ... IF NOT EXISTS`).
   - Must include `# Errors` specifying migration error conditions.
6. **Model, DTO & Value Object Contract**:
   - Struct doc comments must declare the layer: Wire Request DTO (`deny_unknown_fields`), Wire Response DTO, Presentation DTO, or Flattened DB View Row.
   - Database join view rows (e.g. `CategoryHierarchyRow`) must document the canonical column projection order.
   - Value Objects must document "Parse, Don't Validate" rules, trimming behavior, and fallible constructor errors (`try_new`).
7. **Error Enum Contract**:
   - Top-level doc comment must describe the domain failure taxonomy and HTTP status mapping.
   - Every enum variant must carry a doc comment explaining the exact failure condition that triggers it.
   - Methods (`action()`, `code()`) must be documented.
8. **Unadorned Domain Language (Zero Fluff & Promotional Terminology)**:
   - Doc comments describe entities and operations using clean, unadorned domain language (`family`, `currency`, `member`, `account`).
   - Pompous qualifiers and advertising fluff (`initial`, `primary`, `default`, `fixed base currency`) are strictly forbidden across module summaries, item descriptions, and parameter docs.
   - Distinct creation vs update documentation: document creation-time configuration on creation contracts; omit immutable fields from update contracts.

---

## 3. Canonical Templates & Examples

### 3.1 Module-Level Template (`//!`)

```rust
//! Category taxonomy and hierarchy configuration REST route handlers.
//!
//! Exposes HTTP endpoints under `/config` for exploring the 3-tier category hierarchy,
//! retrieving curated palette colors, authoring categories and subcategories, and resetting
//! defaults. Read-only endpoints allow public access for dashboards and visual breakdowns,
//! while administrative mutations require authenticated sessions.
//!
//! # Architecture & CQS Design
//! - **Unified Route Namespace**: Mounted under `/config`, providing canonical entry points
//!   like `GET /api/v1/config/hierarchy` and `POST /api/v1/config/hierarchy/reset` alongside
//!   granular taxonomy resources (`/config/categories/*`).
//! - **Command-Query Separation (CQS)**: Mutation and reset endpoints return lean acknowledgement
//!   envelopes (`ApiResponse<()>`) or created items rather than duplicating expensive hierarchy queries,
//!   eliminating read amplification across high-frequency write paths.
//! - **Strict Error Envelopes**: Handlers delegate persistence to atomic database routines,
//!   mapping domain validation failures and constraint collisions to structured error envelopes.
```

### 3.2 HTTP Route Handler Template (`///`)

```rust
/// Resets all categories, transaction types, and palette colors back to system defaults.
///
/// Canonical route: `POST /api/v1/config/hierarchy/reset`
/// Aliases: `POST /api/v1/config/hierarchies/reset`, `POST /api/v1/config/categories/reset`
///
/// Requires authentication. Atomically clears user-modified categories, types, and colors,
/// re-seeding canonical defaults from the embedded JSON template within a single transaction.
/// In accordance with Command-Query Separation (CQS), returns `ApiResponse<()>` without
/// fetching the updated hierarchy. Callers fetch `GET /api/v1/config/hierarchy` when needed.
///
/// # Security & Access Control
/// - **Auth Requirement**: Authenticated operator session context.
/// - **Role Authorization**: Public / Member / Admin as required.
/// - **Resource Scoping**: Global (unscoped) or tenant-scoped via `family_id`.
///
/// # Ingress
/// - `State(state)`: Application state with shared database connection pool [`DbPool`].
/// - `user`: Authenticated operator session context [`AuthUser`].
/// - `Path(id)`: Target entity 64-bit integer identifier.
/// - `Json(payload)`: Inbound request DTO with `#[serde(deny_unknown_fields)]`.
///
/// # Returns
/// - `Ok(Json(ApiResponse<()>`): 200 OK with restoration confirmation (`data: null`).
/// - Side Effects: Deletes user customizations and re-seeds canonical defaults.
///
/// # Errors
/// - `401 Unauthorized`: Session token missing or expired.
/// - `403 Forbidden`: Actor lacks required administrative role.
/// - `500 Internal Server Error`: [`AppError::ShouldNotBeHappening`] if database execution fails.
async fn reset_defaults(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ApiResponse<()>>, AppError> { ... }
```

### 3.3 Single-Shot Database Query Template (`///`)

```rust
/// Creates a new category scoped under a parent transaction type.
///
/// Validates the category name Value Object, inserts into `categories` calculating
/// the next sort order in a single atomic SQL statement, and returns the created category.
///
/// # Execution Model
/// Executes a single atomic `INSERT` statement directly against [`DbPool`] computing
/// `sort_order` via `(SELECT COALESCE(MAX(sort_order), 0) + 1 FROM categories WHERE type_id = ?)`.
/// Engine-level SQLite constraint errors are classified via [`is_unique_violation`] and
/// [`is_foreign_key_violation`] without open multi-round-trip transactions.
///
/// # Ingress
/// - `pool`: Reference to the shared [`DbPool`].
/// - `payload`: Inbound [`CreateCategoryRequest`] containing target type ID and category name.
///
/// # Returns
/// - `Ok(CategoryItem)` representing the newly created category with generated ID.
///
/// # Errors
/// - Returns [`CategoryError::EmptyCategoryName`] if category name fails Value Object validation.
/// - Returns [`CategoryError::TypeNotFound`] if the parent transaction type does not exist.
/// - Returns [`CategoryError::CategoryAlreadyExists`] if a category with the same name exists under this type.
/// - Returns [`AppError::ShouldNotBeHappening`] on underlying database execution failure.
pub(in crate::features::categories) async fn create_category(
    pool: &DbPool,
    payload: CreateCategoryRequest,
) -> Result<CategoryItem, AppError> { ... }
```

### 3.4 Schema & DDL Migration Template (`///`)

```rust
/// Initializes the category taxonomy schema, indexes, and denormalized hierarchy view.
///
/// Provisions the relational structures for the 3-tier financial category taxonomy:
/// palette colors (`colors`), root transaction types (`transaction_types`), intermediate
/// categories (`categories`), leaf subcategories (`subcategories`), and the flattened
/// read projection `v_category_hierarchy`.
///
/// # Domain Rules & Referential Integrity
/// - **Hierarchical Cascade**: Deleting a transaction type cascades through its child categories
///   and subcategories (`ON DELETE CASCADE`). Deleting a category cascades to its subcategories.
/// - **Color Protection**: Deleting a palette color is restricted (`ON DELETE RESTRICT`) if any
///   transaction type currently references it.
/// - **Scoped Uniqueness**: Root transaction type names are globally unique (`UNIQUE(type_name)`),
///   while category and subcategory names are scoped to their immediate parent (`UNIQUE(type_id, category_name)`
///   and `UNIQUE(category_id, subcategory_name)`).
/// - **Fast Hierarchy Traversal**: Dedicated foreign-key indexes (`idx_categories_type_id`,
///   `idx_subcategories_category_id`) optimize parent-child joins and cascading deletes.
/// - **Pre-Sorted Denormalized View**: The `v_category_hierarchy` view pre-joins types, categories,
///   subcategories, and color hexes ordered by hierarchy sort orders for read queries.
///
/// # Execution & Idempotency
/// - Executes atomically within the caller-provided [`Transaction`].
/// - Idempotent across restarts using `CREATE ... IF NOT EXISTS` DDL.
/// - Each object creation is tracked via [`create_db_object`] under granular action tokens
///   (`CONFIG.CATEGORIES.INIT_SCHEMA.*`) for precise error pinpointing.
///
/// # Errors
/// Returns [`AppError::InitSchema`] if any DDL statement fails to execute.
pub(crate) async fn init_category_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), AppError> { ... }
```

### 3.5 Domain Value Object Template (`///`)

```rust
/// Validated category name Value Object ("Parse, Don't Validate").
///
/// Guarantees that empty or whitespace-only category names cannot be represented
/// in the domain model. Trims surrounding whitespace upon construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct CategoryName(String);

impl CategoryName {
    /// Trims the input and validates non-emptiness.
    ///
    /// # Errors
    /// Returns [`CategoryError::EmptyCategoryName`] with `action` if the trimmed string is empty.
    pub(super) fn try_new(
        raw: impl Into<String>,
        action: &'static str,
    ) -> Result<Self, CategoryError> { ... }

    /// Borrows the validated inner name string slice.
    pub(super) fn as_str(&self) -> &str { ... }

    /// Unwraps and consumes into the owned name [`String`].
    pub(super) fn into_inner(self) -> String { ... }
}
```

---

## 4. Verification & Linting Checklist

Before declaring feature or documentation work complete:
- [ ] Every `.rs` file has an outer module doc comment (`//!`) with a summary and architectural bullets.
- [ ] Every route handler specifies canonical routes, aliases, `# Ingress`, `# Returns`, `# Errors`, and CQS guarantees.
- [ ] Every database function documents its execution model (single-shot atomic vs transaction), `# Ingress`, `# Returns`, and `# Errors`.
- [ ] Every schema migration documents `# Database Objects Created`, `# Invariants`, and `# Errors`.
- [ ] Every Value Object documents validation behavior and constructors.
- [ ] Every error variant has a clear doc comment explaining its trigger condition.
- [ ] Doc-tests and intra-doc links compile cleanly (`./dev.sh backend test`).
- [ ] `./dev.sh backend flint` passes with 0 warnings.
