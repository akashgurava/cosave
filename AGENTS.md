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
1. **Design Pitch**: Propose 4 distinct UX design archetypes on the ticket (e.g. Card Grid, Master-Detail, Interactive Graph, Financial Cockpit).
2. **Approval**: Maintainer approves direction.
3. **Interactive Switcher**: On a `feat/<feature>-ui-prototype` branch, build an interactive switcher at the top of the route rendering all 4 variants against `mock.ts` and `types.ts`. All code is frontend-only; write zero backend code.
4. **Completion Criterion**: Switcher renders cleanly on `:5172`, `./dev.sh check` passes with 0 errors, and maintainer designates the winning archetype on the ticket.

### Phase 2: Backend SSOT & Wire-up (Full Stack)
1. **Contract**: The frozen `frontend/src/lib/features/<feature>/types.ts` is the authoritative backend contract.
2. **Backend Feature**: Implement `backend/src/features/<feature>/` (`models.rs` matching `types.ts`, SQL queries in `db.rs`, HTTP handlers in `routes.rs`, mount in `mod.rs`).
3. **Wire-up & Cleanup**: Replace mock data with `apiFetch<T>()` in `api.ts`, remove the prototype switcher, leaving only the winning design.
4. **Completion Criterion**: All endpoints return `ApiResponse<T>`, `./dev.sh full` passes with 0 errors/warnings across backend and frontend, and a two-axis `/code-review` is conducted.

---

## 3. Feature-First Structure & Backend Standards

Feature code is co-located into symmetrical modules:
- Backend: `backend/src/features/<feature>/` (`mod.rs`, `db.rs`, `models.rs`, `routes.rs`)
- Frontend: `frontend/src/lib/features/<feature>/` (`components/`, `api.ts`, `types.ts`, `mock.ts`)

### Backend Invariants
- **Zero SQL in Routes**: Route handlers only parse HTTP requests, check auth, call `db.rs`, and return `ApiResponse<T>`. All SQL queries and transactions live exclusively in `db.rs`.
- **Visibility**: Default to private visibility; elevate to `pub(crate)` only for items needed across crate modules.
- **API Envelope**: REST responses wrap data in the standard `ApiResponse<T>` envelope with typed `Code` and `Status`.
- **Auth & Passwords**: Hash credentials exclusively via Argon2id with random salts.

---

## 4. Frontend Standards

### UI Primitives & shadcn-svelte
- **Primitives are Vendor Code**: Primitives in `frontend/src/lib/components/ui/` are official upstream components. Install them exclusively via `./dev.sh ui shadcn <component>`. Compose them within feature components; never edit primitives directly.
- **Prototyping Session**: In `DEV=true`, default to an active mock admin user so feature exploration routes remain accessible without auth walls.

### TypeScript & Styling
- **Strict Typing**: Zero `any`. Use `unknown` with runtime guards. Prefer `null` over `undefined` for empty state.
- **Canonical Tailwind v4**: Use parentheses tokens `border-(--border-subtle)` and `size-8` shorthand.
- **Bundle Budget**: Dynamically import heavy libraries (e.g. Apache ECharts) from deep subpaths to keep chunk sizes well below 500 kB.

---

## 5. Tooling & Workflows

Always use [`./dev.sh`](./dev.sh) for dependency management and verification:
- `backend add <crate>` / `ui add <pkg>`: Add dependencies (never edit manifest files manually).
- `dev`: Start backend (`:5171`) and frontend (`:5172`) with proxying and hot reload.
- `check`: Run `cargo check` and `svelte-check`.
- `test`: Run backend unit tests and frontend Vitest suites.
- `flint`: Auto-format and lint both backend and frontend.
- `full`: Complete verification pipeline (test -> check -> build -> flint).

---

## 6. Context Pointers

Load these reference documents on demand when triggered:

- [`CONTEXT.md`](CONTEXT.md): Domain glossary and ubiquitous language. Trigger: when naming entities, creating models/tables, or checking domain definitions.
- [`docs/agents/issue-tracker.md`](docs/agents/issue-tracker.md): GitHub issue conventions and PR branch workflows. Trigger: when creating, reading, commenting on issues, or managing git branches.
- [`docs/agents/triage-labels.md`](docs/agents/triage-labels.md): Issue label mapping. Trigger: when triaging issues or assigning role labels.
- [`docs/agents/domain.md`](docs/agents/domain.md): Domain doc consumer guidelines. Trigger: when exploring codebase architecture or checking ADR conflicts.
