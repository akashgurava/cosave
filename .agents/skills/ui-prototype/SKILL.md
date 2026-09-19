---
name: ui-prototype
description: Prototype interactive frontend UI with 2–3 minimalist archetypes on throwaway mock data before freezing UX. Triggers on prototyping a page or feature, wireframing, or executing Phase 1.
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
1. Consult [`CONTEXT.md`](../../CONTEXT.md) to align terminology with the domain's ubiquitous language.
2. If the request has ambiguous requirements or missing edge states, ask 1 round of 2–3 targeted questions with recommended defaults. Skip if the request is already self-contained.

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
   - `types.ts`: Domain models and discriminated unions.
   - `mock.ts`: Realistic, throwaway sample data.
   - `components/Variant1.svelte`, `Variant2.svelte`, (optional) `Variant3.svelte`.
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
