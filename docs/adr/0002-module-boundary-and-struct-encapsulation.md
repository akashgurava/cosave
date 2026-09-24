# Subsystem Boundaries, Struct Property Encapsulation, and Visibility Hierarchy

Subsystem boundaries enforce single-entrypoint facades at the folder level, struct properties are strictly private with explicit accessors/consumers, and crate visibility is locked down with `#![deny(dead_code)]`.

## Context & Decision

Direct leakage of internal submodule paths across feature and core boundaries coupled with unencapsulated struct fields (`pub` and `pub(crate)`) degraded code searchability, broke encapsulation guarantees, and allowed dormant dead code to accumulate under `#[allow(dead_code)]`.

We established four architectural standards:
1. **Folder Boundary Facades**: Crossing a subsystem boundary (`core/`, `features/<feature>/`) requires callers to import strictly through the root folder name (`core::create_db_object`, `categories::init_schema`). Internal submodules (`mod db;`, `mod models;`) are private to the folder; `mod.rs` acts as the sole public facade. External callers never reach into internal submodules (`core::db::create_db_object` is forbidden).
2. **Strict Struct Property Encapsulation**: All struct fields are private. Direct field visibility (`pub` or `pub(crate)`) is forbidden. External access is provided strictly through:
   - Borrowed reference getters (`item.name() -> &str`, `state.db() -> &DbPool`, `item.created_at() -> i64`).
   - Move consumers for owned decomposition (`payload.into_parts(...)`, `payload.into_name()`).
   - Construction via explicit `new(...)` constructors or builder methods (`with_subcategories(...)`).
3. **Visibility Hierarchy**:
   - `pub`: Reserved strictly for errors (`AppError`, `CategoryError`, `AuthError`), their error query methods (`action()`, `code()`), and essential framework runtime constructs (`DbPool`, `AppState`, `ApiResponse`, `Cli`).
   - `pub(crate)`: Applied exclusively to domain models, DTOs, and functions exposed across features through folder facades.
   - Private / `pub(super)`: Internal submodules, query implementations, and helpers inside a folder.
4. **Zero Dead Code**: Crate root enforces `#![deny(dead_code)]`. `#[allow(dead_code)]` annotations are forbidden. Unused structs, fields, and functions are deleted immediately.
5. **Grouped Import Hierarchy**: All `use` statements reside at the top of the file before item declarations. Imports follow a strict 4-tier hierarchy separated by single blank lines: (1) `std::*`, (2) third-party external crates, (3) `crate::*` internal modules, and (4) `super::*` local subsystem items.

## Consequences

- Cross-folder refactoring is decoupled: modifying internal file structures within `core/` or `features/<feature>/` does not break external callers as long as `mod.rs` preserves the facade.
- Struct invariants are preserved: fields cannot be mutated or bypassed externally, and changes to internal representations do not break call sites relying on accessor methods.
- Compiler guarantees active maintenance: any dead model or dangling function immediately fails `backend check`.
- Errors remain fully public domain contracts with stable `action()` and `code()` APIs.
- Import structure and provenance are immediately scannable across all backend files without inline import noise.
