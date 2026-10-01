# AGENTS.md — CoSave Engineering Standards & Operating Manual

Engineering standards, architectural invariants, and autonomous workflows for **CoSave**.

---

## 1. Tech Stack & Environment

- **Backend**: Rust (Axum, Tokio, Tower-HTTP, Serde, SQLx / SQLite, Argon2id)
- **Frontend**: SvelteKit 2 + Svelte 5 (Runes: `$state`, `$derived`, `$effect`, `$props`), Vite, Tailwind CSS v4, shadcn-svelte, `@sveltejs/adapter-static`
- **Package Management**: `pnpm` (run via `./dev.sh ui`)
- **Automation**: [`./dev.sh`](./dev.sh) for dev, check, test, build, and linting workflows

---

## 2. Architecture & The Two-Phase Lifecycle

The Rust backend is the authoritative **Single Source of Truth (SSOT)** for data, validation, and domain calculations. The frontend is a reactive presentation-layer mirror.

Features are developed in two strictly separated phases to prevent bloated PRs:

### Phase 1: UI Prototyping & UX Freeze (Frontend Only)

Follow [`/ui-prototype`](.agents/skills/ui-prototype/SKILL.md) to explore and freeze frontend UI before writing backend code:

1. **Clarify & Pitch**: Ask 1 round of targeted questions, then pitch 2–3 distinct structural UX archetypes in plain English.
2. **Approval Gate**: Maintainer confirms direction; agent commits to building all 2–3 alternatives with a live switcher.
3. **Interactive Prototypes**: In `frontend/src/lib/features/<feature>/` (or `components/features/<feature>/`) and a sample route, build all 2–3 switchable prototypes against `mock.ts` with Apple/IKEA OLED minimalism (no filler text, unadorned labels, discuss proposals before editing code). All code is frontend-only; write zero backend code.
4. **Completion Criterion**: All prototypes render cleanly on `:5172`, `./dev.sh all check` passes with 0 errors, and maintainer designates the winning archetype on the ticket.

### Phase 2: Backend SSOT & Wire-up (Full Stack)

1. **Contract**: Follow [`/ui-prototype`](.agents/skills/ui-prototype/SKILL.md) Step 6 to scaffold `api.ts` and `api.test.ts` matching frozen `types.ts`. Running `./dev.sh ui test` passes green against `MemoryTransportAdapter` as the executable contract specification.
2. **Backend TDD (Red)**: Author the 3-tier backend test suite test-first: (1) Tier 1 pure domain unit tests for Value Objects and domain math in `models.rs`, (2) Tier 2 direct database and rollback tests in `db/tests.rs` (if transactions or complex queries exist), and (3) Tier 3 black-box Axum route integration tests via `TestApp` in `backend/tests/api/` covering the 3-axis matrix (happy path & UX, domain validation, auth boundary) matching the contract. Tests fail red because handlers/tables are not yet implemented.
3. **Backend Feature (Green)**: Implement `backend/src/features/<feature>/` (`models.rs` matching `types.ts`, SQL queries in `db.rs`, HTTP handlers in `routes.rs`, mount in `mod.rs`) until `./dev.sh backend test` turns green.
4. **Wire-up & Cleanup**: Replace mock data with `api.ts` in the feature view. Never treat responses as blindly asserted JSON (`as T`). Remove the prototype switcher, leaving only the winning design.
5. **Completion Criterion**: All endpoints return `ApiResponse<T>`, all endpoints satisfy the 3-axis black-box test matrix, Vitest feature contract tests pass (`./dev.sh ui test feature <feature>`), live API integration tests pass (`./dev.sh ui test:integration`), `./dev.sh all audit` passes with 0 errors/warnings across backend and frontend, and a two-axis `/code-review` is conducted.

---

## 3. Feature-First Structure & Backend Standards

Feature code is co-located into symmetrical modules:

- Backend: `backend/src/features/<feature>/` (`mod.rs`, `db.rs` or `db/`, `models.rs`, `routes.rs`, `error.rs`)
- Frontend: `frontend/src/lib/features/<feature>/` (`components/`, `api.ts`, `types.ts`, `mock.ts`)

### Backend Invariants

- **Zero SQL in Routes**: Route handlers only parse HTTP requests, check auth, call `db`, and return `ApiResponse<T>`. All SQL queries and transactions live exclusively in `db.rs` or `db/` submodules.
- **Granular Database Actions & Isolation**: Never execute multiple queries, statements, migrations, or error instantiations under a shared or ambiguous action token. Every distinct SQL execution, table creation, index creation, sort calculation, transaction boundary, validation check, and error branch must have its own dedicated call with a globally unique, compile-time `action` token (e.g. `FEATURE.WORKFLOW.STEP[.BRANCH]`). Use `create_db_object` for DDL and `DbResultExt` (`.db_context(action)`) / `db_err` for runtime queries, rolling up into `AppError::ShouldNotBeHappening`. Never leak raw SQL or database internals to client error envelopes.
- **Folder Boundary Facades & Symmetrical Contracts**: Crossing a subsystem boundary (`core/`, `features/<feature>/`) requires callers to import strictly through the root folder name (`core::create_db_object`, `categories::init_schema`). Folder `mod.rs` encapsulates internal submodules (`mod db;`) and exposes the public subsystem API. All domain features implement symmetrical facade contracts: `pub(crate) async fn init_schema(pool: &DbPool) -> Result<(), AppError>`, `pub(crate) fn router() -> Router<AppState>`, and `pub use error::<Feature>Error;`. Never reach into internal submodules across folder seams.
- **Struct Property Encapsulation**: All struct fields are private. Expose data through reference getters (`item.name() -> &str`, `state.db() -> &DbPool`), consume owned payloads via move methods (`payload.into_parts(...)`), and construct instances via `new(...)` constructors or builder methods. Never declare `pub` or `pub(crate)` fields on structs.
- **Visibility Hierarchy & Lowest Visibility First**: Reserve `pub` strictly for errors (`AppError`, feature errors), their query methods (`action()`, `code()`), and essential framework runtime constructs (`DbPool`, `AppState`, `ApiResponse`, `Cli`). Feature models and DTOs shared across features are `pub(crate)`. Folder submodules and internal helpers are private or `pub(super)`. Crate root `lib.rs` declares `mod core; mod features;` as private; never expose subsystem folders as `pub mod`. Callers in `main.rs` import required structs directly from root `cosave`.
- **Zero Dead Code**: Enforce `#![deny(dead_code)]` crate-wide. Delete unused structs, variants, and functions immediately; never annotate with `#[allow(dead_code)]`.
- **Error Token SSOT & Manual Display**: Error types are rigidly typed, feature-scoped, and identifiable by a single unique screaming token defined in `self.code() -> &'static str`. Deriving `thiserror::Error` or duplicating error code strings in format macros is forbidden. Every variant carries a globally unique compile-time `action: &'static str` pinpointing the exact failure site. `std::fmt::Display` is implemented manually, prefixing messages with `{code}. ACTION: {action}`. `std::error::Error` is implemented manually. Feature errors roll up into central `AppError` (`core/error.rs`) via manual `From` implementations. Route handlers return `Result<impl IntoResponse, AppError>`. Failure envelopes return `ErrorPayload { action, message }` in `data` with specific, actionable messages.
- **Server Error Logging SSOT & Zero Call-Site Duplication**: `into_response()` is the authoritative server-side logging sink for all errors (`tracing::warn!` for 4xx, `tracing::error!` for 5xx, structured with `action`, `code`, and `%self`). Never log errors at call sites prior to returning `Err(...)`. Standalone log statements (startup, lifecycle, informational, debug where no error is returned) must carry a dedicated, globally unique compile-time action token prefixed in the message (`{ACTION}. {message}`) following the `FEATURE.WORKFLOW.STEP[.BRANCH]` taxonomy.
- **CLI Feature Isolation & Strict Execution (One-Way Highway)**: CLI parsing (`clap`) is strictly gated behind the non-default `cli` feature; the library compiles without CLI dependencies. Environment and runtime inputs enforce strict, unbending representations: exact matches only (e.g. `"DEV"` or `"PROD"`, never case-insensitive or loose aliases), with uniform fallback semantics. Verification commands run dual passes (without and with `--features cli`).
- **Grouped Import Hierarchy**: All `use` statements reside at the top of the file before item declarations. Group imports into 4 tiers separated by a single blank line: (1) `std::*`, (2) third-party external crates, (3) `crate::*` internal modules, and (4) `super::*` local subsystem items. Never scatter inline `use` declarations inside function bodies unless strictly prevented by conditional compilation.
- **API Envelope**: REST responses wrap data in the standard `ApiResponse<T>` envelope with typed `Code` and `Status`.
- **Auth & Passwords**: Hash credentials exclusively via Argon2id with random salts.
- **3-Tier Backend Testing Architecture & Black-Box HTTP Matrix**: Backend testing follows a 3-tier pyramid with strict mandatory-when-present thresholds and zero vanity tests: (1) **Tier 1 (Pure Domain Unit)** in `models.rs` tests Value Object validation, boundary parsing, and financial math at microsecond speed; (2) **Tier 2 (Direct DB & Transactions)** in `db/tests.rs` or `db.rs` tests multi-statement transaction rollbacks on failure, foreign key cascades (`ON DELETE CASCADE|RESTRICT`), and complex join views directly against an isolated `DbPool` (prohibited for duplicate basic CRUD); (3) **Tier 3 (Black-Box HTTP Route Tests)** in `backend/tests/api/` executes through Axum via `tower::Service::oneshot(Request)` (`TestApp` harness), asserting raw wire JSON envelopes (`serde_json::Value`) across the 3-axis matrix (happy path, domain validation, auth boundary). Route handlers must never be invoked directly in-memory, and testing third-party derives (`serde` roundtrips, getters) is strictly banned. Follow [`docs/backend/agents/testing.md`](docs/backend/agents/testing.md).
- **Database Schema & Transaction Boundaries**: Tables and indexes must use idempotent DDL (`CREATE ... IF NOT EXISTS`) and declare explicit foreign key cascades (`ON DELETE CASCADE|RESTRICT`). Multi-statement write workflows must execute inside explicit `pool.begin().await` transactions with dedicated action tokens (`TX_BEGIN`, step queries, `TX_COMMIT`). Heavy computations (Argon2id, I/O) are strictly forbidden inside open transaction blocks.
- **Domain Invariants & Value Objects ("Parse, Don't Validate")**: Eliminate primitive obsession by wrapping core domain primitives in private Rust newtypes (`CategoryName`, `AmountCents`). Value Objects validate and trim in fallible constructors (`try_new(raw, action) -> Result<Self, FeatureError>`); invalid domain state is unrepresentable. Financial quantities are stored and computed strictly as integer cents (`i64`), never floating-point types (`f32`/`f64`). User entities use prefixed string IDs (`usr_...`). All other persistent domain entities (`colors`, `transaction_types`, `categories`, `subcategories`, `families`, `members`, `accounts`, etc.) standardize on 64-bit integer primary keys (`INTEGER PRIMARY KEY AUTOINCREMENT` in SQLite, `i64` in Rust). System metadata and declarative seed state are tracked via `app_meta` (`key TEXT PRIMARY KEY`, `value TEXT`, `updated_at INTEGER`). Timestamps are stored as `INTEGER NOT NULL` (UTC epoch seconds) via centralized helper `core::time::now_epoch_secs()`.
- **Request DTO Strictness & Three-Tier Model Separation**: Request structs must derive `Deserialize` with `#[serde(deny_unknown_fields)]` and trim inputs. Maintain strict three-tier separation: Wire DTOs != Domain Entities != DB Rows. Raw database rows deriving `sqlx::FromRow` must never be serialized directly into API responses.
- **Resource Authorization & Scoped Queries**: Queries targeting user- or family-owned resources must scope the owner identifier (`family_id` or `user_id`) in their `WHERE` clause. Unscoped mutations and deletions are strictly forbidden. Unauthorized or non-existent resources return 404 Not Found to prevent leaking resource existence across tenants. Privileged endpoints verify capabilities via explicit actor role checks.
- **Startup Configuration SSOT & Immutable AppState**: Environment variables are validated and parsed into an immutable `AppConfig` struct strictly once during application startup. Dynamic `std::env::var` lookups inside route handlers, domain logic, or database operations are strictly forbidden.
- **Authoritative Rustdoc & Contract Documentation Standards**: Rustdoc comments are living contract specifications. Every Rust source file begins with an outer module-level doc comment (`//!`) with an active summary and bulleted architectural highlights. Every public and internal item carries structured doc comments (`///`) with standard Markdown sections (`# Ingress`, `# Returns`, `# Errors`, `# Security & Access Control`, `# Invariants`). Route handlers specify canonical routes, aliases, and CQS guarantees (e.g. `ApiResponse<()>` without read amplification). Database functions document execution models (single-shot atomic vs transaction) and engine-level constraint classification. Doc comments must be updated in lockstep with code refactorings; stale doc comments are treated as compiler bugs. Follow [`docs/backend/agents/documentation.md`](docs/backend/agents/documentation.md).

---

## 4. Frontend Standards

### UI Primitives & shadcn-svelte

- **Primitives are Vendor Code**: Primitives in `frontend/src/lib/components/ui/` are official upstream components. Install them exclusively via `./dev.sh ui shadcn <component>` (which automatically passes `-y` and `-o`/`--overwrite`). Compose them within feature components; never edit primitives directly.
- **Prototyping Session**: In `DEV=true`, default to an active mock admin user so feature exploration routes remain accessible without auth walls.

### TypeScript Rigidity & Compiler Invariants

- **Rust-Grade Compiler Strictness**: `tsconfig.json` enforces `noUncheckedIndexedAccess: true` (indexing `arr[i]` returns `T | undefined`), `noImplicitReturns: true`, `noFallthroughCasesInSwitch: true`, and `noImplicitOverride: true`. Callers must explicitly check for `undefined` before accessing indexed values. Follow [`docs/frontend/agents/compiler-and-types.md`](docs/frontend/agents/compiler-and-types.md).
- **Zero `any`**: Strictly banned across all product and test code. Use `unknown` with runtime type predicates or schema decoders.
- **Zero Blind `as T` Assertions**: Blind type casting (`(await res.json()) as User`) is prohibited. Data crossing any boundary must be validated via runtime decoders (`parseX(raw: unknown)`). The only allowed `as` is `as const` for literal value declarations and narrowing in test assertions.
- **No Implicit Truthy/Falsy Coercion**: Bare boolean checks (`if (count)`, `if (name)`) are forbidden. Always use explicit comparisons: numbers `count > 0` or `count !== 0`, strings `str.length > 0` or `str !== ""`, nullability `val !== null && val !== undefined` or nullish coalescing `??`.
- **Nominal Branding (Rust Newtypes in TypeScript)**: Prevent structural equivalence bugs between IDs and financial integers using compile-time branding via `Brand<T, Tag>` (e.g. `UserId`, `CategoryId`, `AmountCents`). Wrap primitives via validated constructors (`toAmountCents(raw)`).

### Runtime Contract Enforcement ("Parse, Don't Validate")

- **Zero-Dependency Runtime Schema Decoders**: Network and boundary payloads are received as `unknown` and validated through pure-TypeScript decoders (`parseX(raw: unknown): T`) defined in `frontend/src/lib/features/<feature>/types.ts`. Decoders verify types, field presence, and array items, returning frozen structures (`Object.freeze(...)`). Contract violations throw typed `ContractViolationError(message, raw)`. Follow [`docs/frontend/agents/runtime-contracts.md`](docs/frontend/agents/runtime-contracts.md).
- **Three-Tier Model Separation**: Maintain strict separation: Wire DTOs != Presentation Entities != Store State.
- **Structured Error Contracts**: Backend error envelopes (`data: { action, message }`) are parsed into `ApiError` instances exposing `.action` and `.message` directly for targeted user messaging and error badges.

### Svelte 5 Class-Based Rune Stores & State Architecture

- **Class-Based Encapsulation in `.svelte.ts`**: Reactive state is encapsulated in domain store classes in `.svelte.ts` files, mirroring Rust's `struct` + `impl`. Internal reactive variables use native private fields (`#state = $state(...)`). Consumers read state exclusively through read-only getters (`get state()`, `get items()`). Mutations occur strictly through explicit action methods (`async load()`, `create(...)`). Direct external mutations are forbidden. Follow [`docs/frontend/agents/rune-stores.md`](docs/frontend/agents/rune-stores.md).
- **Impossible States Are Unrepresentable (`AsyncState<T>`)**: Asynchronous query states are modeled strictly as tagged discriminated unions (`AsyncState<T> = Idle | Loading | Success<T> | Error<E>`). Never manage detached boolean flags (`isLoading`, `isError`, `data`). Svelte templates perform exhaustive branching on `status`.
- **No `$effect` State Derivation**: Never use `$effect` to synchronize or compute derived state. Use `$derived` for synchronous calculations and explicit action methods for state transitions.

### 3-Tier Frontend Testing Architecture & Zero Vanity Tests

- **3-Tier Testing Pyramid**: (1) **Tier 1 (Pure Domain & Decoder Unit)** in `types.test.ts` or `domain.test.ts` tests `parseX`, branded constructors, and financial math at microsecond speed with zero DOM; (2) **Tier 2 (Rune Store & Contract Tests)** in `store.test.ts` and `api.test.ts` tests state machine transitions and verifies API contracts against `MemoryTransportAdapter`; (3) **Tier 3 (Black-Box User Journey Tests)** in `frontend/e2e/` runs Playwright against real browser flows querying accessible roles and text. In addition, **Live API Integration Tests** (`*.integration.test.ts` via `./dev.sh ui test:integration`) test full-stack HTTP roundtrips against live Axum using `FetchTransportAdapter`, preventing mock drift. Follow [`docs/frontend/agents/testing.md`](docs/frontend/agents/testing.md).
- **Zero Shallow DOM Snapshots**: Banned completely. Never use `toMatchSnapshot()` on rendered markup.
- **Zero Internal Component Mocking**: Never mock internal child components or store internals.

### Styling & Bundle Budget

- **Canonical Tailwind v4**: Use parentheses tokens `border-(--border-subtle)` and `size-8` shorthand.
- **Bundle Budget**: Dynamically import heavy libraries (e.g. Apache ECharts) from deep subpaths to keep chunk sizes well below 500 kB.

---

## 5. Tooling & Workflows

Always use [`./dev.sh`](./dev.sh) for dependency management and verification. CoSave enforces strict target-first syntax (`./dev.sh <target> <action>`):

- `backend add <crate>` / `ui add <pkg>`: Add dependencies (never edit manifest files manually).
- `ui shadcn <component>`: Install shadcn-svelte component (auto-passes `-y` and `-o/--overwrite`).
- `ui node <args...>` / `ui exec <cmd...>` / `backend cargo <args...>`: Run adhoc commands in component context.
- `dev`: Start backend (`:5171`) and frontend (`:5172`) with proxying and hot reload (or `./dev.sh all dev`).
- `serve`: Start production Axum server serving compiled static frontend SPA (or `./dev.sh all serve`).
- `curl <path> [opts]`: Query running backend (:5171) or frontend (:5172) via curl (or `./dev.sh all curl`, `./dev.sh backend curl`; auto-resolves port, shortcuts like `health` -> `/api/v1/health`).
- `ui capture [url]`: Capture desktop and mobile screenshots for `/ui-review` into `.scratch/ui-review/`.
- `all check`: Run compiler and type checks (`./dev.sh backend check` and `./dev.sh ui check`).
- `all test` (or `backend test` / `ui test`): Run unit and contract test suites.
- `ui test:integration` (or `all test:integration`): Run live full-stack API integration tests against active backend (:5171) or ephemeral test server (:5199).
- `all flint`: Auto-format and lint code with fixes applied (fmt + clippy --fix + prettier + eslint --fix + shellcheck).
- `all fbuild`: Fast build gate (`flint` -> `check` -> `build`).
- `all audit`: Complete verification pipeline (`test` -> `test:integration` -> `check` -> `build` -> `flint --no-fix`).
- `smoke test`: Run container HTTP API integration suite.
- `doctor`: Check local toolchain prerequisites.

### Target-Scoped Verification Invariant

When making changes exclusively to backend code (or frontend code), run target-specific verification commands (`./dev.sh backend check|test|flint` or `./dev.sh ui check|test|flint`). Do NOT trigger full-workspace runs (`./dev.sh all audit`, `./dev.sh all flint`) on incremental, single-tier edits. Reserve `all audit` exclusively for full-stack completion gates.

### Server Execution & Occupied Ports Invariant

When starting development or production servers (`./dev.sh dev` or `./dev.sh serve`), if ports (`:5171`, `:5172`) are occupied, the maintainer is already running the server outside in their host terminal or IDE. Never attempt to kill or terminate occupying processes. `./dev.sh` detects this, reports the existing instance, and returns cleanly. Assume the server is healthy and active: proceed directly to query endpoints via `./dev.sh curl <endpoint>`, capture screenshots via `./dev.sh ui capture`, or run UI/backend verification against the live server.

---

## 6. Context Pointers

Load these reference documents on demand when triggered:

- [`CONTEXT.md`](CONTEXT.md): Domain glossary and ubiquitous language. Trigger: when naming entities, creating models/tables, or checking domain definitions.
- [`docs/shared/domain.md`](docs/shared/domain.md): Domain doc consumer guidelines. Trigger: when exploring codebase architecture or checking ADR conflicts.
- [`docs/backend/agents/errors.md`](docs/backend/agents/errors.md): Backend error creation, propagation, logging standards, and API response formatting. Trigger: when creating or editing error types, implementing route handlers, mapping database failures, or adding log statements.
- [`docs/backend/agents/encapsulation.md`](docs/backend/agents/encapsulation.md): Backend subsystem encapsulation, struct property accessors, and visibility hierarchy. Trigger: when creating structs, defining module exports, or managing visibility.
- [`docs/backend/agents/testing.md`](docs/backend/agents/testing.md): Backend API black-box testing standards, TestApp harness, and 3-axis test matrix. Trigger: when authoring or reviewing backend tests, or validating route contracts.
- [`docs/backend/agents/domain-and-data.md`](docs/backend/agents/domain-and-data.md): Database schema DDL, transactions, Value Objects, parse-don't-validate, and money/timestamp SSOT. Trigger: when creating database tables, writing transactions, handling money/timestamps, or creating domain models and value objects.
- [`docs/backend/agents/ingress-and-security.md`](docs/backend/agents/ingress-and-security.md): Request DTO hygiene, 3-tier models, actor-scoped queries, RBAC, and immutable config. Trigger: when creating request DTOs, implementing authorization checks, scoping database queries, or accessing application configuration.
- [`docs/backend/agents/documentation.md`](docs/backend/agents/documentation.md): Authoritative Rustdoc standards, outer module documentation (`//!`), route contracts, DB queries, and section formatting. Trigger: when writing doc comments, documenting modules/routes/queries/models, or editing documentation.
- [`docs/frontend/agents/compiler-and-types.md`](docs/frontend/agents/compiler-and-types.md): Frontend compiler strictness, type invariants, nominal branding, and banned loose TS patterns. Trigger: when authoring TypeScript, defining types/models, or configuring TS.
- [`docs/frontend/agents/runtime-contracts.md`](docs/frontend/agents/runtime-contracts.md): Pure-TS runtime schema decoders, parse-don't-validate at boundaries, and error contract handling. Trigger: when creating DTOs, decoding API responses, or writing transport adapters.
- [`docs/frontend/agents/rune-stores.md`](docs/frontend/agents/rune-stores.md): Svelte 5 class-based rune stores, private state encapsulation, and discriminated union async state machines. Trigger: when authoring state management, handling async operations, or building feature stores.
- [`docs/frontend/agents/testing.md`](docs/frontend/agents/testing.md): Frontend 3-tier testing standards, live API integration tests against live Axum, contract testing against MemoryTransportAdapter, and banned vanity snapshots. Trigger: when writing frontend tests, API integration tests, or Playwright tests.
- [`.agents/skills/ui-prototype/SKILL.md`](.agents/skills/ui-prototype/SKILL.md): Prototype frontend UI archetypes. Trigger: when prototyping a page or feature, wireframing, or executing Phase 1.
- [`.agents/skills/ui-review/SKILL.md`](.agents/skills/ui-review/SKILL.md): Audit UI, UX, and type rigidity. Trigger: when reviewing UI quality, auditing rendered pages, or grading frontend code rigor.
