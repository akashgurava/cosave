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
2. **Backend TDD (Red)**: Write Axum route integration tests in `backend/src/features/<feature>/routes.rs` matching the contract. Tests fail red because handlers/tables are not yet implemented.
3. **Backend Feature (Green)**: Implement `backend/src/features/<feature>/` (`models.rs` matching `types.ts`, SQL queries in `db.rs`, HTTP handlers in `routes.rs`, mount in `mod.rs`) until `./dev.sh backend test` turns green.
4. **Wire-up & Cleanup**: Replace mock data with `api.ts` in the feature view. Never treat responses as blindly asserted JSON (`as T`). Remove the prototype switcher, leaving only the winning design.
5. **Completion Criterion**: All endpoints return `ApiResponse<T>`, Vitest feature contract tests pass (`./dev.sh ui test feature <feature>`), `./dev.sh all audit` passes with 0 errors/warnings across backend and frontend, and a two-axis `/code-review` is conducted.

---

## 3. Feature-First Structure & Backend Standards

Feature code is co-located into symmetrical modules:
- Backend: `backend/src/features/<feature>/` (`mod.rs`, `db.rs` or `db/`, `models.rs`, `routes.rs`, `error.rs`)
- Frontend: `frontend/src/lib/features/<feature>/` (`components/`, `api.ts`, `types.ts`, `mock.ts`)

### Backend Invariants
- **Zero SQL in Routes**: Route handlers only parse HTTP requests, check auth, call `db`, and return `ApiResponse<T>`. All SQL queries and transactions live exclusively in `db.rs` or `db/` submodules.
- **Granular Database Actions & Isolation**: Never execute multiple queries, statements, or migrations under a single action token. Every distinct SQL execution, table creation, index creation, sort calculation, and transaction boundary must have its own dedicated call with a unique, compile-time `action` token. Use `create_db_object` for DDL and `DbResultExt` (`.db_context(action)`) / `db_err` for runtime queries, rolling up into `AppError::ShouldNotBeHappening`. Never leak raw SQL or database internals to client error envelopes.
- **Folder Boundary Facades**: Crossing a subsystem boundary (`core/`, `features/<feature>/`) requires callers to import strictly through the root folder name (`core::create_db_object`, `categories::init_schema`). Folder `mod.rs` encapsulates internal submodules (`mod db;`) and exposes the public subsystem API. Never reach into internal submodules across folder seams.
- **Struct Property Encapsulation**: All struct fields are private. Expose data through reference getters (`item.name() -> &str`, `state.db() -> &DbPool`), consume owned payloads via move methods (`payload.into_parts(...)`), and construct instances via `new(...)` constructors or builder methods. Never declare `pub` or `pub(crate)` fields on structs.
- **Visibility Hierarchy**: Reserve `pub` strictly for errors (`AppError`, feature errors), their query methods (`action()`, `code()`), and essential framework runtime constructs (`DbPool`, `AppState`, `ApiResponse`, `Cli`). Feature models and DTOs shared across features are `pub(crate)`. Folder submodules and internal helpers are private or `pub(super)`.
- **Zero Dead Code**: Enforce `#![deny(dead_code)]` crate-wide. Delete unused structs, variants, and functions immediately; never annotate with `#[allow(dead_code)]`.
- **Strict Error Rigidity**: Error types are rigidly typed, feature-scoped, and identifiable by a single unique screaming token (`self.code()`). Every variant carries compile-time `action: &'static str`. Feature errors roll up into central `AppError` (`core/error.rs`) via `#[from]`. Route handlers return `Result<impl IntoResponse, AppError>`. Failure envelopes return `ErrorPayload { action, message }` in `data` with specific, actionable messages.
- **Grouped Import Hierarchy**: All `use` statements reside at the top of the file before item declarations. Group imports into 4 tiers separated by a single blank line: (1) `std::*`, (2) third-party external crates, (3) `crate::*` internal modules, and (4) `super::*` local subsystem items. Never scatter inline `use` declarations inside function bodies unless strictly prevented by conditional compilation.
- **API Envelope**: REST responses wrap data in the standard `ApiResponse<T>` envelope with typed `Code` and `Status`.
- **Auth & Passwords**: Hash credentials exclusively via Argon2id with random salts.

---

## 4. Frontend Standards

### UI Primitives & shadcn-svelte
- **Primitives are Vendor Code**: Primitives in `frontend/src/lib/components/ui/` are official upstream components. Install them exclusively via `./dev.sh ui shadcn <component>` (which automatically passes `-y` and `-o`/`--overwrite`). Compose them within feature components; never edit primitives directly.
- **Prototyping Session**: In `DEV=true`, default to an active mock admin user so feature exploration routes remain accessible without auth walls.

### TypeScript & Styling
- **Rust-Grade Type Rigidity**: Zero `any`. Use `unknown` with runtime guards. Prefer `null` over `undefined` for empty state. Model variants as tagged discriminated unions. Exported functions, utilities, and store actions must declare explicit return types.
- **Runtime Contract Enforcement (No Blind JSON Asserts)**: Frontend API layers must NEVER treat network responses as blindly cast JSON (`as T`). Every API response must be validated through runtime schema decoders (`schema: parseX` in `types.ts`) that strictly verify types, variants, and nullability, mirroring Rust's `serde_json::from_str::<T>()`. Contract violations throw typed `ContractViolationError`.
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
- `all flint`: Auto-format and lint code with fixes applied (fmt + clippy --fix + prettier + eslint --fix + shellcheck).
- `all fbuild`: Fast build gate (`flint` -> `check` -> `build`).
- `all audit`: Complete verification pipeline (`test` -> `check` -> `build` -> `flint --no-fix`).
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
- [`docs/agents/domain.md`](docs/agents/domain.md): Domain doc consumer guidelines. Trigger: when exploring codebase architecture or checking ADR conflicts.
- [`docs/agents/backend-errors.md`](docs/agents/backend-errors.md): Backend error creation, propagation, and API response formatting. Trigger: when creating or editing error types, implementing route handlers, or mapping database failures.
- [`docs/agents/backend-encapsulation.md`](docs/agents/backend-encapsulation.md): Backend subsystem encapsulation, struct property accessors, and visibility hierarchy. Trigger: when creating structs, defining module exports, or managing visibility.
- [`.agents/skills/ui-prototype/SKILL.md`](.agents/skills/ui-prototype/SKILL.md): Prototype frontend UI archetypes. Trigger: when prototyping a page or feature, wireframing, or executing Phase 1.
- [`.agents/skills/ui-review/SKILL.md`](.agents/skills/ui-review/SKILL.md): Audit UI, UX, and type rigidity. Trigger: when reviewing UI quality, auditing rendered pages, or grading frontend code rigor.
