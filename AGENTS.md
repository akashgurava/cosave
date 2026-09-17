# AGENTS.md — CoSave Coding Standards & Guidelines

This document defines the engineering standards and architectural principles for **CoSave**. All contributors to this codebase must adhere strictly to these rules.

---

## 1. Tech Stack

- **Backend**: Rust 1.98+ (Axum 0.8, Tokio 1.53, Tower-HTTP 0.7, Serde 1.0, SQLx 0.8 / SQLite, Argon2 0.5)
- **Frontend**: SvelteKit 2 + Svelte 5 (runes mode: `$state`, `$derived`, `$effect`, `$props`), Vite 8, Tailwind CSS v4 (`@tailwindcss/vite`), `@sveltejs/adapter-static` (SPA fallback)
- **Package Manager**: `pnpm` (never `npm` or `npx` for package management)
- **Containerization**: Multi-stage Dockerfile (`node:24-alpine` -> `rust:alpine` -> `alpine:3.21`)
- **Workflow Automation**: [`./dev.sh`](./dev.sh)

---

## 2. Core Architectural Principle: Backend as Single Source of Truth

The Rust backend is the authoritative **Single Source of Truth (SSOT)** across the entire application:
1. **Data**: The backend owns all schemas, models, database persistence, and validation integrity.
2. **API Routes**: All route definitions, request contracts, parameter schemas, and response envelopes originate from and are enforced by the backend.
3. **Business Logic**: All domain calculations, financial rules, spend aggregations, savings metrics, and workflows reside exclusively in Rust backend services. Business logic must never be duplicated or independently executed in frontend code.
4. **Frontend as an Interchangeable Mirror**: The frontend serves strictly as a reactive, presentation-layer mirror of backend state and contracts. Its role is focused on UI rendering, user interaction, navigation, and accessibility.

### Feature Development Workflow: UI-Driven Demand, Backend-First Implementation
All contributors and AI agents must adhere to this development lifecycle:
1. **Demand Originates from the UI**: Features are conceived through direct user experience on the web interface. The user notices a functional need or UI element and requests implementation from the AI assistant.
2. **Implementation is Strictly Backend-First**:
   - Never implement a feature by writing client-side business logic, mocking state, or persisting data client-side.
   - **Step 1 (Backend Data & Logic)**: Define/update database schemas in SQLite (`backend/src/db.rs`), Rust domain models (`backend/src/models/`), and business calculations in Rust services.
   - **Step 2 (API Contract)**: Expose clean, strongly-typed RESTful endpoints under `/api/v1/*` using standardized `ApiResponse<T>` envelopes.
   - **Step 3 (Frontend Presentation)**: Bind the frontend to the new backend endpoints via `apiFetch<T>()` and render the backend-authoritative data in cohesive Svelte components.
3. **Headless Server & Swappable Frontend Principle**: The API server is completely decoupled and frontend-agnostic. The UI is a flexible client that can be refactored, redesigned, or replaced (e.g. mobile app, alternative web framework, desktop, CLI) without altering backend domain logic or persistence.

---

## 3. Rust Standards

### Visibility First Principle
1. **Default to Private**: Start with private visibility for **all** structs, struct members/fields, functions, methods, enums, and constants.
2. **Elevate Conservatively**:
   - Only elevate to `pub(crate)` when a type or method must be shared across modules within the binary.
   - Only elevate to `pub` if the item is explicitly exported in a public library interface or required by an external framework/trait boundary.
3. **No Unneeded `pub`**: Never mark struct fields `pub` if accessor methods or crate-internal visibility suffices.

### API Contract & Serialization
- All API responses must use the standardized envelope:
  ```rust
  pub(crate) struct ApiResponse<T: Serialize> {
      pub(crate) code: Code,
      pub(crate) status: Status,
      pub(crate) data: T,
  }
  ```
- **Response Code**: Must use the `Code` enum (e.g. `Code::Zero` serialized as integer `0`), with constructor helpers such as `Code::zero()`.
- **Response Status**: Must use the `Status` enum (serialized as `SCREAMING_SNAKE_CASE` string, e.g. `"HEALTHY"`, `"OK"`), with constructor helpers such as `Status::healthy()`.
- **No Test Overhead**: Avoid writing trivial tests that only verify third-party library behavior (e.g., verifying that Serde serializes `0` to `0`). Focus tests on real domain logic and integration contracts.

### Database & Persistence (SQLx & SQLite)
- Manage connections through asynchronous connection pools (`sqlx::SqlitePool`) with WAL journal mode and foreign keys enabled.
- All schema initialization and migrations reside exclusively in backend services (`backend/src/db.rs`).
- Passwords must be hashed using Argon2id with cryptographically secure random salts; never log or persist plaintext credentials.
- Application state is shared via Axum's type-safe `AppState` extractor (`axum::extract::State`).

### Logging
- Default logging level is `info` (`cosave=info,tower_http=info`).
- Debug logging is activated via command line flags (`-v`, `--verbose`, `--debug`) or the `RUST_LOG` environment variable.
- In Docker containers, logging runs at `info` level by default.

---

## 4. TypeScript & Frontend Standards

### Strict Typing is the Essence
1. **Explicit Types for Everything**: Define explicit `type`, `interface`, or typed enum/const objects for all data models, API payloads, state variables, and function signatures.
2. **Zero `any` Policy**:
   - `any` is prohibited.
   - Use `unknown` with runtime type narrowing guards for dynamic data.
3. **Avoid `undefined`**:
   - Avoid `undefined` wherever possible. Prefer `null` or explicit discriminated union states unless omitting optional properties significantly reduces boilerplate without loss of safety.
   - Specify explicit default values (e.g. `details: unknown = null`) instead of optional `undefined`.
4. **Helper Functions & Extraction**:
   - Build dedicated helper functions for extraction, parsing, and data conversion.
   - Throw explicit, strongly-typed errors (e.g. `UnanticipatedCodeError`, `UnanticipatedStatusError`, `ApiError`) when an API returns unhandled codes or statuses.
   - Centralize API calls through typed fetch utilities like `apiFetch<T>()`.

### UI & Styling (Tailwind CSS v4)
- Use canonical Tailwind v4 class syntax:
  - Use `border-(--border-subtle)` instead of `border-[var(--border-subtle)]`.
  - Use `bg-(--bg-surface)` instead of `bg-[var(--bg-surface)]`.
  - Use `bg-(image:--brand-gradient)` instead of `bg-[image:var(--brand-gradient)]`.
  - Use shorthand utilities: `size-8` instead of `w-8 h-8`.
- Follow class ordering rules enforced by `prettier-plugin-tailwindcss` and `eslint-plugin-tailwindcss`.
- Verify canonical class compliance via:
  ```bash
  ./dev.sh ui check
  ```

### SvelteKit & Svelte 5 Conventions
- Use Svelte 5 Runes mode:
  - `$state()` for reactive variables.
  - `$derived()` for computed values.
  - `$effect()` for side effects and lifecycle subscriptions.
- SvelteKit 2 Routing & Navigation:
  - SPA mode enabled via `prerender = false; ssr = false;` in `+layout.ts` with `adapter-static` fallback to `index.html`.
  - Internal links must use `resolve(...)` from `$app/paths` (enforced by `svelte/no-navigation-without-resolve`).

### Component Decomposition Principle
1. **Logical Isolation Over Raw Reuse**:
   - Extract UI sections into standalone components whenever they represent a logical, self-contained structure (e.g. `TopNav.svelte`, `Sidebar.svelte`, `Footer.svelte`, `StatusBadge.svelte`, cards, or toolbars).
   - Reusability is **not** a prerequisite for component extraction. Even if a section is used only once (such as a top navigation bar in a root layout), extracting it isolates concerns, keeps layouts and routes clean and high-level, and simplifies testing and refactoring.
2. **Directory & Structure Standards**:
   - Shared and structural components belong in `frontend/src/lib/components/`.
   - Expose components via barrel exports in `frontend/src/lib/components/index.ts` (e.g. `export { default as TopNav } from "./TopNav.svelte";`).
   - Use the `$components` alias configured in `svelte.config.js` to import components cleanly:
     ```typescript
     import { TopNav } from "$components";
     ```
   - Keep page routes (`+page.svelte`) and layouts (`+layout.svelte`) as orchestrators of cohesive child components rather than monolithic templates with inline section markup.
   - Component props must use explicit TypeScript interfaces with Svelte 5 `$props()`.

---

## 5. Development & CI Workflow

**Strict Requirement**: Always prefer and use [`./dev.sh *`](./dev.sh) for standard workflows instead of running ad-hoc `cargo` or `pnpm` commands (e.g. use `./dev.sh ui test` rather than `pnpm test`). Only run workflows for the specific component modified (e.g. `./dev.sh ui fbuild` when touching frontend; `./dev.sh backend fbuild` when touching backend; `./dev.sh fbuild` only when modifying both):

### Component-Scoped Workflows
- **Backend (Rust)**:
  ```bash
  ./dev.sh backend fbuild       # Fast build: check -> flint (auto-fixes) -> build (release)
  ./dev.sh backend full [--fix]  # Full pipeline: test -> check -> build -> flint (fails fast)
  ./dev.sh backend flint [--fix] # Formats and lints backend
  ./dev.sh backend check        # Runs cargo check
  ./dev.sh backend test         # Runs cargo test (accepts extra arguments)
  ./dev.sh backend lint         # Runs cargo fmt --check and clippy (-D warnings)
  ./dev.sh backend lint --fix   # Auto-fixes formatting and clippy warnings
  ./dev.sh backend format       # Runs cargo fmt
  ./dev.sh backend build        # Compiles backend (e.g. ./dev.sh backend build --release)
  ./dev.sh backend dev          # Starts Axum server with verbose debug logging
  ./dev.sh backend serve        # Starts Axum production server in release mode
  ./dev.sh backend add <crate>  # Adds crate dependency via cargo
  ```

- **UI / Frontend (SvelteKit 2 + Tailwind v4)**:
  ```bash
  ./dev.sh ui fbuild            # Fast build: check -> flint (auto-fixes) -> build
  ./dev.sh ui full [--fix]      # Full pipeline in order: test -> check -> build -> flint (fails fast)
  ./dev.sh ui flint [--fix]     # Formats and lints frontend
  ./dev.sh ui check             # Runs svelte-check and canonical Tailwind class check
  ./dev.sh ui test              # Runs Vitest unit tests (accepts extra arguments)
  ./dev.sh ui lint              # Runs svelte-check, canonical classes, ESLint, Prettier
  ./dev.sh ui lint --fix        # Auto-fixes Tailwind classes, ESLint, and Prettier
  ./dev.sh ui format            # Auto-formats via canonical Tailwind and Prettier
  ./dev.sh ui build             # Builds SvelteKit static SPA into dist/
  ./dev.sh ui dev               # Starts Vite dev server on :5173
  ./dev.sh ui serve             # Previews compiled static SPA via Vite preview
  ./dev.sh ui add <pkg>         # Adds package dependency via pnpm
  ```

### Full-Stack Workflows
- **Development Server**:
  ```bash
  ./dev.sh dev [target]         # Concurrently starts backend (:3000) and frontend (:5173) with live reload
  ```
- **Production Server**:
  ```bash
  ./dev.sh serve [local|docker] # Runs production server (supports --build to compile fresh SPA)
  ```
- **Fast Build Pipeline (No server, no unit tests)**:
  ```bash
  ./dev.sh fbuild [target]      # Runs check -> flint (auto-fixes) -> build for backend, ui, or all
  ```
- **Full Verification Pipeline**:
  ```bash
  ./dev.sh full [ui] [--fix]    # Runs test -> check -> build -> flint sequentially
  ```
- **Format & Lint (Flint)**:
  ```bash
  ./dev.sh flint                # Formats and lints both backend and frontend
  ./dev.sh flint --fix          # Auto-fixes formatting and lints both components
  ```
- **Lint & Type Check**:
  ```bash
  ./dev.sh lint                 # Checks both backend and frontend
  ./dev.sh lint --fix           # Fixes both backend and frontend
  ```
- **Format**:
  ```bash
  ./dev.sh format               # Formats both backend and frontend
  ```
- **Check**:
  ```bash
  ./dev.sh check                # Runs cargo check and svelte-check
  ```
- **Build**:
  ```bash
  ./dev.sh build [target]       # Builds backend, ui, docker, or all (default: all)
  ```
- **Automated Tests & Container Smoke Test**:
  ```bash
  ./dev.sh test                 # Runs cargo test, Vitest, and validates live container endpoints
  ```

---

## 6. Don'ts

- **Don't** add `pub` visibility to items unless crate-internal (`pub(crate)`) or external export is strictly required.
- **Don't** implement or duplicate business logic, domain calculations, or state authority on the frontend; always rely on the backend as the single source of truth.
- **Don't** build UI-requested features using client-only state, mocks, or local storage; always model and implement the backend database schema, business logic, and REST API contract first.
- **Don't** introduce `any` types in TypeScript code.
- **Don't** write verbose CSS in `<style>` blocks when Tailwind utility classes and design tokens suffice.
- **Don't** use arbitrary bracket syntax `[var(--...)]` in Tailwind v4 when canonical parentheses `(--...)` are supported.
- **Don't** run ad-hoc `cargo` or `pnpm` commands (e.g. `pnpm test`, `pnpm run check`) when a `./dev.sh` subcommand exists.
- **Don't** leave background dev server processes running after exit.
