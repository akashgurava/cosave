# CoSave Specification

> [!IMPORTANT]
> **Scope**: Focus exclusively on bootstrap specifications defined in `HANDOFF.md` and `TODO.md`.

---

## 1. System Purpose & Domain Model

CoSave is a self-hosted, local-first financial management service distributed as a Docker container running a single static binary.

- **Architecture (Jellyfin Model)**:
  - **API Server**: Headless HTTP API powering all domain operations.
  - **Web UI**: Embedded default web client decoupled from the server core.
- **Domain Scope**: Families track aggregated financial accounts, monitor member cash flow, and categorize transactions.
- **Tenancy Boundary**: Multi-tenant domain model where all persistent core entities are partitioned strictly by `family_id`.
- **Data Boundary**: Local persistence with zero third-party telemetry.

---

## 2. Agent Behavioral Protocol & Collaboration Standards

All AI agents operating on this repository must adhere strictly to these operational protocols:

- **Relentless Invariant Defense & Technical Challenge**: Stress-test directives, probe failure modes, and challenge architectural shortcuts against documented repository invariants. When challenging a directive: (1) cite the invariant name and file location, (2) describe the exact structural defect caused, and (3) propose a compliant alternative. Await explicit maintainer confirmation on architectural decisions before modifying code.
- **The Two-Attempt Debugging Circuit Breaker**: If a compiler diagnostic or test failure persists after two consecutive attempts, halt execution immediately. Revert trial edits (`git restore .`) and report: (1) observable symptom, (2) the two rejected hypotheses with failure evidence, and (3) the core structural tension requiring maintainer input.
- **Read-Only Inquiries & Scoped Answering**: Treat diagnostic, conceptual, and architectural questions as read-only. Deliver targeted analysis in chat first, answering exclusively what is asked without unprompted tutorials or scope expansion. Code modifications require an explicit, imperative directive.
- **Technical Objectivity & User-Driven Direction**: Anchor interaction strictly in code correctness and invariant enforcement, omitting flattery, conversational filler, and passive agreement. The maintainer drives project direction; assist directly without assuming unconfirmed intent or taking unprompted initiative.
- **Atomic Cadence & Step-by-Step Reporting**: Execute multi-step tasks in single atomic increments. Report progress, state the proposed next action, and await maintainer confirmation before advancing.
- **Specification Hygiene**: Formal specifications and living documentation capture verified requirements and structural invariants only. Brainstorming, tentative explorations, and conversational nuance must never leak into specifications as generic disclaimers or meta-commentary.

---

## 3. Workspace Topology & Crate Boundaries

Rust Cargo workspace root manifests `crates/*`, package namespace `cosave-<crate>`:

- `crates/core` (`cosave-core`):
  - **Targets**: Native + `wasm32-unknown-unknown`.
  - **Role**: Pure domain leaf. Zero I/O, OS, or runtime dependencies.
  - **Internal Deps**: None.
  - **Owns**: Value Objects, strongly-typed domain enums with Serde renaming (`#[serde(rename_all = "snake_case")]`), scale-aware minor-unit money math, nominal ID branding (`FamilyId`, `AccountId`), Wire DTO schemas, and domain error taxonomy (`CoreError`).
  - **Invariants**: Validates input-intrinsic rules via fallible constructors (`ValueObject::try_new(raw, action)`). Invalid domain state is unrepresentable.
- `crates/db` (`cosave-db`):
  - **Target**: Native.
  - **Role**: Dedicated persistence engine.
  - **Internal Deps**: `cosave-core`.
  - **Owns**: Storage schemas, migrations, seed routines, relational integrity (uniqueness, foreign keys, cascade rules), state-dependent invariants (balance availability), atomic transactions, connection pooling, and private `<Entity>Row` storage models.
  - **Invariants**: All queries targeting tenant data must scope `family_id` in their `WHERE` clause; unscoped queries violate multi-tenant isolation. Storage models and engine driver types are capped at `pub(crate)` and remain internal to `crates/db`.
- `crates/api` (`cosave-api`):
  - **Target**: Native binary.
  - **Role**: Server host executable and transport adapter.
  - **Internal Deps**: `cosave-core`, `cosave-db`.
  - **Owns**: Headless HTTP transport, routing, request/response serialization (`ApiResponse<T>`), HTTP status mapping, cookie/session management, authenticated actor resolution (`user_id`, `family_id`), distribution asset serving, and server error logging sink.
  - **Invariants**: Transport adapter only. Extracts authenticated actor context, invokes database operations using `core` types, and maps results to HTTP envelopes. Route handlers contain zero SQL queries or business logic.
- `crates/ui` (`cosave-ui`):
  - **Target**: `wasm32-unknown-unknown`.
  - **Role**: Client web application and presentation mirror.
  - **Internal Deps**: `cosave-core`.
  - **Owns**: Reactive DOM state, route navigation, user input buffers, and client-side view state.
  - **Invariants**: Presentation mirror only. Imports `core` Value Objects natively via WebAssembly to execute domain validation client-side before network dispatch without duplicating domain validation logic.

**Invariants**:

- **DAG Boundary**: Dependency graph is strictly inward (`api -> {db, core}`, `db -> core`, `ui -> core`). Introducing lateral, cyclical, or reverse crate dependencies is an architectural invariant violation.
- **WASM Gate**: Pure platform-agnostic domain logic only; introducing OS or platform assumptions into `cosave-core` is an invariant violation. `cargo check -p cosave-core --target wasm32-unknown-unknown` must pass unconditionally.

---

## 4. Data Contracts & The Three-Tier Model

Every domain rule, schema, and operational state is owned by exactly one crate. Duplicating rules or validation logic across layers is an architectural defect.

### The Three-Tier Model Quarantine

Data crossing crate seams maps strictly through three tiers:
`Wire DTO (core) <-> Domain Entity (core) <-> Storage Record (db)`

1. **Tier 1 — Wire DTOs (`crates/core::<feature>::dto`)**:
   - **Boundary**: External network and client-server boundaries.
   - **Rules**: Passive data carriers with zero domain logic. Instantiated via designated `new` constructors or Serde deserialization. Distinct mutation contracts (`CreateXRequest`, `UpdateXRequest`, `XResponse`); immutable properties established at creation are omitted from update DTOs. Deserializers enforce `#[serde(deny_unknown_fields)]` to reject unexpected inputs.

2. **Tier 2 — Domain Entities & Value Objects (`crates/core::<feature>::model`)**:
   - **Boundary**: Shared ubiquitous domain language across all crates.
   - **Rules**: Core domain logic and state validation. Shared natively with `crates/ui` via WASM.

3. **Tier 3 — Storage Rows (`crates/db::<feature>::row`)**:
   - **Boundary**: Strictly internal persistence representations in `crates/db`.
   - **Naming**: `<Entity>Row` (e.g. `FamilyRow`, `AccountRow`, `MemberRow`).
   - **Rules**: Engine-specific table definitions and database row types. Initialized strictly via database driver row decoders or dedicated private `from_row` constructors. Struct visibility is capped at `pub(crate)`. Conversions to domain entities occur through private field access in move methods (`into_domain(self)` or `try_into_domain(self)`). Public storage facades accept and return `crates/core::<feature>::model` entities exclusively; storage rows and driver types never cross crate boundaries.

---

## 5. Repository Engineering Standards

- **Ubiquitous Language**: Canonical entity taxonomy across all layers: `Family`, `Member`, `Account`. Use exact domain terminology across code, database schemas, DTOs, and UI without aliases or synonyms.
- **Opaque Struct Encapsulation**: All struct fields across every crate are strictly private (zero `pub` or `pub(crate)` fields). Direct struct instantiation (`Foo { .. }`) is restricted exclusively to the struct's own constructors (`new`, `try_new`) and decoders. State is accessed via reference getters (`item.name() -> &str`) and consumed via move methods (`item.into_parts(...)`).
- **Feature Facades & Opaque Submodules**: Feature directories (`crates/<crate>/src/<feature>/`) encapsulate internal modules (`dto`, `model`, `row`) as private (`mod dto;`). Never expose internal feature submodules via `pub mod`. The feature `mod.rs` acts as an opaque facade, selectively re-exporting public types (`pub use model::X;`). External callers import strictly from the feature namespace (`cosave_core::<feature>::X`).
- **Grouped Import Hierarchy**: Declare all imports at the top of the file before item declarations. Group `use` statements into four distinct tiers separated by a single blank line: (1) `std::*`, (2) external crates, (3) `crate::*` internal modules, and (4) `super::*` local subsystem items. Never declare inline `use` statements inside function bodies.
- **Visibility Hierarchy & Zero Test Inflation**:
  - **Visibility Gradient**: Default strictly to lowest visibility: private -> `pub(super)` -> `pub(crate)` -> `pub` (reserved exclusively for cross-crate boundaries).
  - **Test Invariant**: Never increase visibility on a function, struct, or field to make a test compile. Test internal logic from within co-located unit tests (`#[cfg(test)] mod tests { use super::*; }`), or strictly through the public facade.
  - **Zero Dead Code**: Delete orphaned functions, unused parameters, and obsolete types immediately upon refactoring. Never adjust visibility or add suppressions to keep unreferenced code compiling.
- **Zero Diagnostic Suppressions**: Do not silence compiler diagnostics with `#[allow(...)]`, `#[allow(dead_code)]`, `#[allow(unused)]`, or `#[ignore]`. Resolve issues at the root cause. Silencing diagnostics with attributes is strictly prohibited by CI verification.
- **Startup Configuration SSOT**: Parse environment variables exclusively within the `AppConfig` startup constructor in `crates/api`. Downstream crates consume runtime configuration strictly through read-only references on `AppState`. Dynamic lookups via `std::env::var` inside route handlers, domain logic, or database operations are strictly prohibited.
- **Cargo Workspace & Dependency Inheritance**:
  - **Centralized Dependencies**: All external dependencies are declared centrally in root `[workspace.dependencies]`; member crates import via `<dep>.workspace = true`. Adding dependencies directly without workspace inheritance is prohibited.
  - **Manifest Inheritance**: Member manifests inherit package metadata (`version`, `edition`, `authors`, `license`) via `[workspace.package]`. Overriding these metadata keys in member crates is prohibited.
- **Action Token Taxonomy & Centralized Error Sink**:
  - **Token Schema**: `{CRATE}.{FEATURE}.{TARGET_ENTITY}.{ACTION_VERB}`
    - `{CRATE}`: `CORE | DB | API | UI` (identifies the originating crate)
    - `{FEATURE}`: `AUTH | FAMILY_CONFIG | CATEGORY_CONFIG | TRANSACTIONS` (identifies the domain area)
    - `{TARGET_ENTITY}.{ACTION_VERB}`: Globally unique step identifier (e.g. `USER_CREDENTIALS.INSERT`, `FAMILY_LIMIT.VALIDATE`, `ACCOUNT_ID.PARSE`).
  - **Centralized Error Sink**: All errors across `crates/core`, `crates/db`, and `crates/api` roll up into the unified `ApiResponse<T>` envelope with `data: { action, message }`. Logging occurs exclusively at the centralized `into_response()` sink in `crates/api`; call sites bubble `Result::Err` directly via `?` without ad-hoc logging.
- **Rustdoc Standards**:
  - **Module Overview (`//!`)**: Every source file begins with an outer doc comment describing what the module is responsible for and how it fits into the crate.
  - **Item Documentation (`///`)**: Every public and `pub(crate)` item must provide:
    - **First Line**: A direct, plain-English summary of what the item does and why it exists.
    - **Body**: Plain explanation of how it works and what callers need to provide.
    - **`# Errors`**: Bulleted list pairing each error variant with its specific trigger condition (e.g. `- Error::DuplicateEmail: If the email address is already taken.`).
    - **`# Panics`**: Explicitly document every execution branch that can panic, or state `# Panics: None`.
  - **Lockstep Updates**: Doc comments are living contract specifications. Updating code without updating its doc comments in the same commit is treated as a compiler defect.

---

## 6. Financial Domain & Money Math Invariants

Financial calculations and currency representations must adhere strictly to these non-negotiable invariants:

- Scale-Aware Minor Units: All monetary amounts across the stack are stored, computed, and transferred strictly as 64-bit signed integer minor units (`AmountMinorUnits` / `i64`).
- Zero Floating-Point Arithmetic: The use of floating-point types (`f32`, `f64`) is strictly forbidden across calculations, database columns, models, and wire DTOs. Financial math must execute exclusively using scale-aware integer operations.
- ISO 4217 Currency Scale Enforcement: Currencies enforce their ISO 4217 decimal scale (scale 0 for currencies like JPY, scale 2 for USD/EUR/INR, scale 3 for KWD). Conversion between display strings and integer minor units must split on decimal characters without floating-point coercion.

---

## 7. Testing Architecture & Verification Standards

Testing must verify logical soundness and guard invariants, never serve as performative vanity metrics:

- Zero Vanity Stat-Padding: Test count is not a metric. Tests exist exclusively to challenge logical soundness, failure boundaries, and edge cases. Trivial tests that merely assert string interpolations, field getters, derive roundtrips (e.g. standard Serde serialization), or raw type casts without domain logic are strictly banned.
- Fault-Seeking Invariant Defense: Tests must actively probe how logic breaks. Write tests to prove that invalid domain states, boundary overflows, malformed payloads, and constraint violations fail fast and cleanly, rather than writing performative "happy path" checks that merely conform to existing code.
- Tier 1 (Pure Domain Unit Tests in `crates/core`): Stress-test Value Object constructors (`try_new`), scale-aware financial math, and parsing boundary conditions at microsecond speed with zero database, networking, or async dependencies.
- Tier 2 (Database Transaction & Integrity Tests in `crates/db`): Verify multi-statement transaction rollbacks on failure, foreign key cascade integrity (`ON DELETE RESTRICT`), and SQLite constraint enforcement directly against an isolated `DbPool`. Prohibited for duplicate basic CRUD.
- Tier 3 (Black-Box HTTP Integration Tests in `crates/api`): Execute requests through Axum via `tower::Service::oneshot(Request)` (`TestApp` harness), asserting raw wire JSON envelopes (`ApiResponse<T>`) across the 3-axis matrix: (1) happy path and status code, (2) rejection of malformed or unauthorized domain inputs, and (3) authentication boundaries. Handler functions must never be called directly in-memory.
- Cross-Tenant Scoping Defense: Integration tests must explicitly provision two separate families (`Family A` and `Family B`) and assert that Family A attempting to read, update, or delete Family B's resource returns `404 Not Found`, verifying that tenant existence is never leaked.
- Raw Wire JSON Assertions: When verifying API responses, assert directly against raw wire JSON (`serde_json::Value`). Never deserialize into Rust response structs during tests, which can mask missing fields, default values, or Serde renaming defects.
- Isolated Deterministic Databases: Every integration test must spin up an independent, isolated database instance via `TestApp::new()`. Tests execute in parallel and must be completely order-independent with zero shared mutable state.
- Idempotency & Repeat Execution: Schema migrations (DDL) and default seed routines must include tests proving they execute safely multiple times without crashing or generating duplicate data (`CREATE TABLE IF NOT EXISTS`, conflict handling).

---

## 8. CLI Orchestrator & Toolchain Protocol

All workspace operations, checks, testing, and lifecycle processes are centralized through the root `./dev.sh` script. Agents must exclusively use `./dev.sh` commands rather than invoking raw `cargo check -p ...`, ad-hoc compilation commands, or scattered manual processes:

- Mandatory Orchestrator Invariant: Never bypass `./dev.sh` to run raw compiler commands. Using `./dev.sh` guarantees that the correct target flags (such as `--target wasm32-unknown-unknown` for `crates/ui`), environment variables, and proxy configurations are applied consistently across all crates.
- CLI Workspace & Dependency Management: Scaffold crates and add dependencies exclusively via official Cargo CLI commands (e.g. `cargo new --lib crates/<name>`, `cargo add -p <crate> <dep>`), never via manual edits to `Cargo.toml`. Add dependencies without version arguments so Cargo resolves latest stable releases. If a CLI command requires interactive terminal input, halt and request maintainer intervention.
- `./dev.sh doctor`: Environment and toolchain verification. Validates installed Rust compilers, Cargo targets (`wasm32-unknown-unknown`), Trunk CLI, SQLite runtime, and workspace directory structure.
- `./dev.sh dev`: Starts the local development environment. Boots the headless Axum backend on port `:5171` and the Trunk Leptos development server on port `:5173` with automatic reverse-proxying of `/api/*` to the backend.
- `./dev.sh check`: Fast workspace type checking. Runs strict compiler checks across all native crates (`crates/core`, `crates/db`, `crates/api`) and automatically checks `crates/ui` against the `wasm32-unknown-unknown` target.
- `./dev.sh test`: Automated test execution. Dispatches Tier 1 pure domain tests, Tier 2 database transaction tests, and Tier 3 black-box API integration tests across the workspace.
- `./dev.sh lint`: Code hygiene and style enforcement. Executes `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` across the workspace to enforce zero compiler warnings.
- `./dev.sh audit`: Full pre-flight verification gate. Sequentially executes `check`, `lint`, and `test` across the entire workspace. Must pass with zero errors and zero warnings before any work is considered complete.
- `./dev.sh build`: Production release compilation. Builds optimized static WebAssembly assets via Trunk in `crates/ui`, then compiles the production `cosave` executable in `crates/api` with embedded static assets for single-binary distribution.
- `./dev.sh curl <endpoint>`: Direct API inspection helper. Formats and sends HTTP requests to the active backend instance (e.g. `./dev.sh curl health` or `./dev.sh curl /api/v1/categories/hierarchy`), handling port resolution and cookie extraction.

---

## 9. Production Packaging & Docker Deployment

The authoritative deployment artifact for CoSave is a self-hosted Docker container running the unified `cosave` executable:

- Single-Binary Container Architecture: The production Docker image packages exclusively the single compiled `cosave` binary. Axum serves both the `/api/v1/*` REST routes and the embedded WebAssembly user interface directly from that single process.
- Multi-Stage Container Build: Built via a multi-stage `Dockerfile`:
  - Builder Stage: Compiles `crates/ui` to static WebAssembly assets via Trunk, then compiles the release `cosave` binary in `crates/api` embedding the assets via `rust-embed`.
  - Runtime Stage: Copies exclusively the compiled `cosave` executable into a `scratch` runtime base.
- Single Port & Volume Contract: The container exposes a single port (`:5172`) and mounts a single persistent host directory (`/data`) containing the SQLite database (`/data/cosave.db`).
- Container Orchestration via `./dev.sh`: Docker build and run workflows are wrapped directly in `./dev.sh` (`./dev.sh docker build`, `./dev.sh docker run`) ensuring consistent port bindings (`-p 5172:5172`) and volume mounts (`-v $(pwd)/data:/data`).
