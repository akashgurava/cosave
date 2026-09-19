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
4. **Completion Criterion**: All prototypes render cleanly on `:5172`, `./dev.sh check` passes with 0 errors, and maintainer designates the winning archetype on the ticket.

### Phase 2: Backend SSOT & Wire-up (Full Stack)
1. **Contract**: The frozen `frontend/src/lib/features/<feature>/types.ts` is the authoritative backend contract.
2. **Backend Feature**: Implement `backend/src/features/<feature>/` (`models.rs` matching `types.ts`, SQL queries in `db.rs`, HTTP handlers in `routes.rs`, mount in `mod.rs`).
3. **Wire-up & Cleanup**: Replace mock data with `api.get/post<T>()` in `api.ts` passing runtime schema decoders (`schema: parseX`). Never treat responses as blindly asserted JSON (`as T`). Remove the prototype switcher, leaving only the winning design.
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
- **Rust-Grade Type Rigidity**: Zero `any`. Use `unknown` with runtime guards. Prefer `null` over `undefined` for empty state. Model variants as tagged discriminated unions. Exported functions, utilities, and store actions must declare explicit return types.
- **Runtime Contract Enforcement (No Blind JSON Asserts)**: Frontend API layers must NEVER treat network responses as blindly cast JSON (`as T`). Every API response must be validated through runtime schema decoders (`schema: parseX` in `types.ts`) that strictly verify types, variants, and nullability, mirroring Rust's `serde_json::from_str::<T>()`. Contract violations throw typed `ContractViolationError`.
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
- [`.agents/skills/ui-prototype/SKILL.md`](.agents/skills/ui-prototype/SKILL.md): Prototype frontend UI archetypes. Trigger: when prototyping a page or feature, wireframing, or executing Phase 1.
- [`.agents/skills/ui-review/SKILL.md`](.agents/skills/ui-review/SKILL.md): Audit UI, UX, and type rigidity. Trigger: when reviewing UI quality, auditing rendered pages, or grading frontend code rigor.
