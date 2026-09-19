---
name: ui-prototype
description: Prototype interactive frontend UI with 2–3 minimalist archetypes on throwaway mock data before freezing UX, and scaffold consumer-driven API contract tests for backend TDD. Triggers on prototyping a page or feature, wireframing, executing Phase 1, or writing feature api.test.ts contract tests.
---

# UI Prototyping & Wireframing

Execute Phase 1 of CoSave's feature lifecycle: explore, iterate, and freeze UI/UX before any backend code is written.

## Core Invariants

1. **Frontend-Only Quarantine**: All edits are strictly quarantined to `frontend/src/lib/features/<feature>/` (or `components/features/<feature>/`) and the prototype route `frontend/src/routes/...`. Write zero Rust code, zero SQL queries, and zero migrations.
2. **Throwaway Mock Data**: Keep all data in `mock.ts`. Never wire real network requests, authentication barriers, or backend endpoints during prototyping.
3. **Apple & IKEA OLED Minimalism**:
   - Monochromatic palette: Pure `#000000` black in dark mode with `border-(--border-subtle)` hairline borders; crisp `#ffffff` gallery white in light mode. Quiet, purposeful accents; zero neon or gradient fills.
   - Unadorned noun labels: Direct names ("Accounts", "Members", "Limit"); omit filler words ("Total accounts", "Active members").
   - Zero patronizing instructional copy: If an action or layout is self-evident, omit explanatory subtitles.
   - Vendor primitives: Compose untouched upstream primitives from `$lib/components/ui/` installed via `./dev.sh ui shadcn <component>`.
4. **Conversational Discipline**: When the user asks for suggestions, proposals, naming options, or trade-offs, discuss in chat first—never proactively edit code. When pitching archetypes, always implement all 2–3 alternatives with a live switcher.

## Process

### Step 1: Clarify Requirements
1. Consult [`CONTEXT.md`](../../CONTEXT.md) to align terminology with the domain's ubiquitous language. If an entity name is fuzzy, overloaded, or warrants a recorded architectural decision, trigger **[`/domain-modeling`](../domain-modeling/SKILL.md)** to sharpen terms and record an ADR before drafting types.
2. If the feature concept requires stress-testing or has open design trade-offs, trigger **[`/grilling`](../grilling/SKILL.md)** (or **[`/grill-me`](../grill-me/SKILL.md)**). Otherwise, ask 1 round of 2–3 targeted questions with recommended defaults. Skip if the request is already self-contained.

> **Completion Criterion**: Ambiguities resolved. Do not pitch archetypes or touch code while core questions remain open.

### Step 2: Pitch Archetypes & Approval Gate
Before writing code, pitch the plan to the user in plain English:
1. **Summary**: 2–3 sentences on entity relationships, data ownership, and available actions.
2. **Archetypes**: Pitch 2 or 3 structurally distinct UX archetypes (e.g. Master-Detail Split, Grouped Cards Canvas, High-Density Table & Drawer) emphasizing spatial layout over design jargon.
3. **Commitment**: State: *"I will implement all [2 or 3] archetypes with a live switcher on [route] so you can compare them in the browser. Does this layout match your expectations, or would you like any adjustments before I build all of them?"*
4. **Gate**: Pause and wait for human confirmation.

> **Completion Criterion**: Human explicitly responds with approval or directional steering. Writing code before this gate is prohibited.

### Step 3: Implement Prototypes with Live Switcher
1. Scaffold feature files in `frontend/src/lib/features/<feature>/`:
   - `types.ts`: Domain models and discriminated unions, shaped via **[`/codebase-design`](../codebase-design/SKILL.md)** (narrow interface surface, deep domain behaviour, clean in-memory transport seam).
   - `mock.ts`: Realistic, throwaway sample data.
   - `components/Variant1.svelte`, `Variant2.svelte`, (optional) `Variant3.svelte`: Compose unstyled upstream primitives. Trigger **[`/shadcn-svelte`](../shadcn-svelte/SKILL.md)** to add, configure, or style shadcn-svelte components.
2. Mount all variants on the sample route (e.g. `frontend/src/routes/configuration/<feature>/+page.svelte`) with a top pill/tab switcher binding to `?variant=1`, `?variant=2`.
3. Verify zero errors: run `./dev.sh check`.

> **Completion Criterion**: All 2–3 variants render cleanly at `http://localhost:5172`, the switcher toggles between them smoothly, and `./dev.sh check` passes with 0 errors.

### Step 4: Iterate on Feedback
1. Direct the user to test the prototypes live in the browser.
2. **Action vs. Proposal Rule**:
   - Concrete directive (e.g. "replace button with icon"): apply the edit immediately in the feature directory.
   - Open question or naming choice: propose 2–4 options in chat with concise rationale; wait for the user's choice before editing code.

> **Completion Criterion**: Human selects the winning archetype or confirms the converged layout.

### Step 5: UX Freeze
1. Remove the switcher and delete losing variant components.
2. Promote the winning variant to the primary feature view wired to `mock.ts`.
3. Freeze `frontend/src/lib/features/<feature>/types.ts` as the authoritative contract for Phase 2 backend implementation.
4. Run `./dev.sh check`.

> **Completion Criterion**: Route renders only the winning design, switcher code is deleted, `./dev.sh check` passes with 0 errors, and `types.ts` is ready for backend handoff.

### Step 6: Executable Contract Specification (`api.ts` & `api.test.ts`)
*Execute immediately following Step 5 UX Freeze to lock the consumer-driven contract before backend implementation.*

Transform the frozen `types.ts` into an executable consumer-driven contract specification:

1. **In-Memory Seam & Zero-Breakage Invariant**:
   - In `frontend/src/lib/features/<feature>/api.test.ts`, install `MemoryTransportAdapter` on `api.setTransport()` in `beforeEach` and restore in `afterEach`.
   - **Zero-Breakage Guarantee**: Tests execute entirely in-process. Running `./dev.sh ui test` passes 100% green immediately without backend dependencies or network flakiness.

2. **Required Coverage Matrix for Every Feature Endpoint**:
   - **Canonical Envelopes**: Mock `{ code: Code.Zero, status: Status.Ok, data: ... }` (or `Status.Healthy`); verify that `api.ts` parses data through the frozen runtime decoder (`schema: parseX`) and returns the strongly-typed domain model.
   - **Schema Rigidity**: Inject malformed payloads (missing fields, bad types, invalid union tags); verify that `ContractViolationError` is thrown, locking UI state against corrupted data.
   - **Typed Error Envelopes**: Mock Axum error responses (400 `BadRequest`, 401 `Unauthenticated`/`InvalidCredentials` with `isUnauthorized: true`, 404 `NotFound`, 409 `UserExists` with `isConflict: true`, 500 `InternalError`, and network drops).
   - **Path & Query Formatting**: Assert that interpolated path parameters (`:id`) and query params serialize and URL-encode correctly.
   - **Store Transitions**: If the feature has a reactive store (`store.svelte.ts`/`store.ts`), verify state transitions and error recovery against the in-memory transport seam (never shallow `vi.spyOn` mocks).

3. **Backend TDD Handoff (Red → Green)**:
   The contract test suite becomes the executable spec for Phase 2:
   1. Backend Axum route integration tests in `backend/src/features/<feature>/routes.rs` are written to mirror this exact contract. Trigger **[`/tdd`](../tdd/SKILL.md)** (or **[`/implement`](../implement/SKILL.md)**) to drive Axum route tests Red first.
   2. Rust tests run **Red** (`cargo test` fails before handlers/tables exist).
   3. Backend queries in `db.rs` and handlers in `routes.rs` are written until tests turn **Green**.
   4. The UI feature view flips from `mock.ts` to `api.ts` with zero contract drift.

> **Completion Criterion**: `frontend/src/lib/features/<feature>/api.ts` and `api.test.ts` exist, cover 100% of feature endpoints, status envelopes, and schema decoders, and `pnpm --dir frontend vitest run src/lib/features/<feature>/api.test.ts` passes with 0 failures.

## Next Flow

- **Audit & Polish**: Trigger **[`/ui-review`](../ui-review/SKILL.md)** to run a 5-axis visual, accessibility, and type-rigidity audit on live screenshots before backend implementation.
- **Backend Implementation**: Trigger **[`/tdd`](../tdd/SKILL.md)** to build the backend endpoints Red-to-Green against the contract specification.

