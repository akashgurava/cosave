---
name: ui-prototype
description: Prototype interactive frontend UI with 2–3 minimalist archetypes on throwaway mock data before freezing UX, and scaffold consumer-driven API contract tests for backend TDD. Triggers on prototyping a page or feature, wireframing, executing Phase 1, or writing feature api.test.ts contract tests.
---

# UI Prototyping & Wireframing

Execute Phase 1 of CoSave's feature lifecycle: explore, iterate, and freeze UI/UX before any backend code is written.

Prototyping is not an excuse for sloppy code. The types, runtime decoders, and mock queries authored during Phase 1 become the permanent, authoritative contract for Phase 2 backend TDD. Build presentation scaffolding with Rust-grade strictness from minute one.

## Core Invariants

1. **Frontend-Only Quarantine**: All edits are strictly quarantined to `frontend/src/lib/features/<feature>/` (or `components/features/<feature>/`) and the prototype route `frontend/src/routes/...`. Write zero Rust code, zero SQL queries, and zero migrations.
2. **Schema Grounding & Database Fidelity**:
   - Before drafting types or generating mock data, inspect `data/cosave.db` via `sqlite3` or check backend migrations (`backend/src/features/*/db/schema.rs`) to ground IDs, types, and foreign keys in existing schema.
   - Reuse existing domain entities and types (`$lib/types/core`, `$lib/features/*/types`). Never invent parallel, conflicting taxonomy or detached ID ranges.
3. **Strict Domain Modeling & Zero "Optional Slop"**:
   - Apply nominal branding (`Brand<number, 'EntityId'>`, `MinorUnits = Brand<number, 'MinorUnits'>`).
   - Author pure runtime schema decoders (`parse<Entity>(raw: unknown): Entity`) in `types.ts` from the very first commit.
   - Model required properties strictly. Distinguish required domain fields, explicitly nullable fields (`null`), and optional query parameters. Avoid blanket optional (`?`) markers on domain entities.
   - Zero `any` and zero unchecked `as T` casting.
4. **Clean JSON Mock Architecture**:
   - Store mock datasets in `mock_<feature>.json` (pure JSON, zero comments).
   - Generate realistic datasets: 30–50 items minimum with real temporal variation, varied statuses, and foreign key IDs matching existing database seeds (`typeId`, `categoryId`, `accountId`, `memberId`).
   - Decode mock JSON through runtime schema decoders in `mock.ts` (`parse<Entity>Response(raw)`). Never write repetitive `as Brand` casts.
5. **Route-Conformant In-Memory Queries**:
   - Mock querying functions in `mock.ts` must mirror real HTTP route query parameters (`startDate`, `endDate`, `searchQuery`, `limit`, `page`, etc.).
   - Map UI presets (e.g. date presets "1D", "7D", "1M") to canonical query boundaries before filtering.
6. **Live Cross-Feature State Integration**:
   - Dynamic presentation attributes (palette hex colors, currency symbols, badges) must be derived from shared authoritative stores (`categoryStore`, `familyStore`) or props, never static snapshots hardcoded on items.
   - Changes made in configuration routes (e.g. updating a type color) must reactively reflect in feature views.
7. **Apple & IKEA OLED Minimalism**:
   - Monochromatic palette: Pure `#000000` black in dark mode with `border-(--border-subtle)` hairline borders; crisp `#ffffff` gallery white in light mode. Quiet, purposeful accents; zero neon or gradient fills.
   - Unadorned noun labels: Direct names ("Accounts", "Members", "Limit"); omit filler words ("Total accounts", "Active members").
   - Zero patronizing instructional copy: If an action or layout is self-evident, omit explanatory subtitles.
   - Vendor primitives: Compose untouched upstream primitives from `$lib/components/ui/` installed via `./dev.sh ui shadcn <component>`.
8. **Conversational Discipline**:
   - When the user asks for suggestions, proposals, naming options, or trade-offs, discuss in chat first—never proactively edit code.
   - When pitching archetypes, always implement all 2–3 alternatives with a live switcher.

## Process

### Step 1: Ground in Existing Schema & Clarify Requirements

1. **Inspect Existing Database & Models**:
   - Inspect active database records in `data/cosave.db` via `sqlite3` (e.g. `sqlite3 data/cosave.db ".schema"` and relevant tables) to record active IDs for types, categories, accounts, and members.
   - Review existing domain types in `frontend/src/lib/types/core.ts` and related feature modules.
2. **Consult Domain Language**:
   - Consult [`CONTEXT.md`](../../CONTEXT.md). If an entity name is fuzzy, overloaded, or warrants a recorded architectural decision, trigger **[`/domain-modeling`](../domain-modeling/SKILL.md)** to sharpen terms and record an ADR before drafting types.
3. **Clarify Open Intent**:
   - If the feature concept requires stress-testing, trigger **[`/grilling`](../grilling/SKILL.md)**. Otherwise, ask 1 round of 2–3 targeted questions with recommended defaults. Skip if self-contained.

> **Completion Criterion**: Real database IDs and schema relationships identified; domain terminology aligned with `CONTEXT.md`; open questions resolved. Zero code written before this gate.

### Step 2: Pitch Archetypes & Approval Gate

Before writing code, pitch the plan to the user in plain English:
1. **Summary**: 2–3 sentences on entity relationships, data ownership, schema grounding, and available actions.
2. **Archetypes**: Pitch 2 or 3 structurally distinct UX archetypes (e.g. Master-Detail Split, Grouped Cards Canvas, High-Density Table & Drawer) emphasizing spatial layout and responsiveness (desktop vs. mobile).
3. **Commitment**: State: *"I will implement all [2 or 3] archetypes with a live switcher on [route] so you can compare them in the browser. Does this layout match your expectations, or would you like any adjustments before I build all of them?"*
4. **Gate**: Pause and wait for human confirmation.

> **Completion Criterion**: Human explicitly responds with approval or directional steering. Writing code before this gate is prohibited.

### Step 3: Implement Prototypes with Live Switcher

1. **Scaffold Strict Domain Types & Decoders**:
   In `frontend/src/lib/features/<feature>/types.ts`:
   - Declare branded identifiers and strict domain interfaces (no blanket optional `?` markers on domain entities).
   - Write pure-TypeScript runtime schema decoders (`parse<Entity>(raw: unknown): Entity`) returning frozen objects (`Object.freeze(...)`).
2. **Create Realistic Mock Dataset & Query Engine**:
   - Author `frontend/src/lib/features/<feature>/mock_<feature>.json` with 30–50 varied items using existing database foreign key IDs. Pure JSON without comments.
   - In `mock.ts`, import the JSON, decode via `parse<Entity>Response(raw)`, and expose unified, route-conformant query/filter functions matching future HTTP query parameters.
3. **Build Switchable Variants**:
   - Build `components/Variant1.svelte`, `Variant2.svelte`, (optional) `Variant3.svelte` composing upstream shadcn primitives. Trigger **[`/shadcn-svelte`](../shadcn-svelte/SKILL.md)** as needed.
   - Derive shared metadata (palette colors, account currency) from live stores (`categoryStore`, `familyStore`) or props.
   - Support responsive layouts (e.g., dedicated compact/touch layouts for mobile `< 768px`).
4. **Mount on Sample Route**:
   - Mount variants on the sample route with a top switcher binding to `?variant=1`, `?variant=2`.
   - In `DEV=true`, default to an active session so exploration routes remain accessible.
5. **Verify Clean**:
   - Run `./dev.sh ui check` and `./dev.sh ui test`.
   - Verify all variants render cleanly at `http://localhost:5172`.

> **Completion Criterion**: All 2–3 variants render cleanly at `http://localhost:5172`, the switcher toggles between them smoothly, and `./dev.sh ui check` passes with 0 errors.

### Step 4: Iterate on Feedback

1. Direct the user to test the prototypes live in the browser.
2. **Action vs. Proposal Rule**:
   - Concrete directive (e.g. "replace button with icon", "center paginator"): apply the edit immediately.
   - Open question, trade-off, or naming choice: propose 2–4 options in chat with concise rationale; wait for user choice before editing code.
3. Keep code strictly conforming to repository standards throughout iterations (no regression into `as any` or loose types).

> **Completion Criterion**: Human selects the winning archetype or confirms the converged layout.

### Step 5: UX Freeze

1. Remove the switcher and delete losing variant components.
2. Promote the winning variant to the primary feature view wired to `mock.ts`.
3. Freeze `frontend/src/lib/features/<feature>/types.ts` as the authoritative contract for Phase 2 backend implementation.
4. Run target-scoped verification: `./dev.sh ui check`, `./dev.sh ui test`, and `./dev.sh ui flint`.

> **Completion Criterion**: Route renders only the winning design, switcher code is deleted, `./dev.sh ui check` and `./dev.sh ui test` pass with 0 errors/failures, and `types.ts` is frozen for backend handoff.

### Step 6: Executable Contract Specification (`api.ts` & `api.test.ts`)

*Execute immediately following Step 5 UX Freeze to lock the consumer-driven contract before backend implementation.*

Transform the frozen `types.ts` into an executable consumer-driven contract specification:

1. **In-Memory Seam & Zero-Breakage Invariant**:
   - In `frontend/src/lib/features/<feature>/api.test.ts`, install `MemoryTransportAdapter` on `api.setTransport()` in `beforeEach` and restore in `afterEach`.
   - **Zero-Breakage Guarantee**: Tests execute entirely in-process. Running `./dev.sh ui test` passes 100% green immediately without backend dependencies or network flakiness.

2. **Required Coverage Matrix for Every Feature Endpoint**:
   - **Canonical Envelopes**: Mock `{ code: Code.Zero, status: Status.Ok, data: ... }` (or `Status.Healthy`); verify that `api.ts` parses data through the frozen runtime decoder (`schema: parseX`) and returns the strongly-typed domain model.
   - **Schema Rigidity**: Inject malformed payloads (missing fields, bad types, invalid union tags); verify that `ContractViolationError` is thrown, locking UI state against corrupted data.
   - **Typed Error Envelopes**: Mock Axum error responses (400 `BadRequest`, 401 `Unauthenticated`/`InvalidCredentials` with `isUnauthorized: true`, 404 `NotFound`, 409 `UserAlreadyExists` with `isConflict: true`, 500 `InternalError`, and network drops).
   - **Path & Query Formatting**: Assert that interpolated path parameters (`:id`) and query params serialize and URL-encode correctly.
   - **Store Transitions**: If the feature has a reactive store (`store.svelte.ts`/`store.ts`), verify state transitions and error recovery against the in-memory transport seam (never shallow `vi.spyOn` mocks).

3. **Backend TDD Handoff (Red → Green)**:
   The contract test suite becomes the executable spec for Phase 2:
   1. Backend Axum route integration tests in `backend/tests/api/` are written to mirror this exact contract. Trigger **[`/tdd`](../tdd/SKILL.md)** to drive Axum route tests Red first.
   2. Rust tests run **Red** (`./dev.sh backend test` fails before handlers/tables exist).
   3. Backend queries in `db.rs` and handlers in `routes.rs` are written until tests turn **Green**.
   4. The UI feature view flips from `mock.ts` to `api.ts` with zero contract drift.

> **Completion Criterion**: `frontend/src/lib/features/<feature>/api.ts` and `api.test.ts` exist, cover 100% of feature endpoints, status envelopes, and schema decoders, and `./dev.sh ui test` passes with 0 failures.

## Next Flow

- **Audit & Polish**: Trigger **[`/ui-review`](../ui-review/SKILL.md)** to run a 5-axis visual, accessibility, and type-rigidity audit on live screenshots before backend implementation.
- **Backend Implementation**: Trigger **[`/tdd`](../tdd/SKILL.md)** to build the backend endpoints Red-to-Green against the contract specification.
