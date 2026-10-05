---
name: ui-review
description: Audit frontend routes from live screenshots and source code across 5 axes using parallel sub-agents. Triggers on reviewing UI/UX quality, auditing rendered pages, or inspecting frontend code rigor.
---

# UI, UX & Frontend Code Review

Audit and elevate frontend routes and feature components across five objective axes by pairing live Playwright screenshots with Svelte source code.

## The 5 Axes

1. **Aesthetics & Visual Hierarchy**: Clear primary focal point; balanced typography scale; monochromatic OLED (pure `#000000` dark mode, `#ffffff` light mode, quiet accents; zero neon or AI gradient tropes).
2. **UX & Shell Navigability**: Zero patronizing instructional subtitles; comfortable touch targets (≥32px); mobile sidebar trigger (`<Sidebar.Trigger />`) permanently accessible on narrow viewports; parent/root routes redirect cleanly; destructive actions protected and styled in red.
3. **Design System Fidelity**: Zero hardcoded hex colors, magic numbers, or margins; canonical Tailwind v4 tokens (`border-(--border-subtle)`, `bg-card`, `text-foreground`, `size-8`); unadorned noun labels ("Accounts", not "Total accounts"); untouched upstream shadcn-svelte primitives. Trigger **[`/shadcn-svelte`](../shadcn-svelte/SKILL.md)** when inspecting, adding, or modifying component primitives.
4. **Accessibility & Semantics**: Keyboard navigability (Tab order, Esc closes dialogs); native `aria-label` on icon-only buttons; WCAG AA contrast (≥4.5:1 text, ≥3:1 borders); form inputs bound to accessible labels.
5. **Resilience, Type Rigidity & Contract Verification**: Graceful empty states and text truncation; responsive collapse at 390px; zero `any` or loose casts; zero blind JSON asserts (`as T`) on network data; mandatory runtime schema decoders (`schema: parseX` mirroring Rust's `serde_json::from_str::<T>()`); strict nullability (`T | null` over `undefined`); tagged discriminated unions; explicit method return types; small interface surface with deep behaviour (**[`/codebase-design`](../codebase-design/SKILL.md)**); partial test fixtures typed without loose `as` casts (**[`/migrate-to-shoehorn`](../migrate-to-shoehorn/SKILL.md)**); **executable consumer-driven contract test suite (`api.test.ts`) verified against `MemoryTransportAdapter` with 100% green pass rate**.

### Scoring Rubric (1 to 5)
- **5 (Exemplary)**: Production-ready, Apple/Stripe-grade polish, zero token/a11y defects, 100% strict Rust-grade typing, complete `api.test.ts` contract suite passing green.
- **4 (Solid)**: Clean and functional; minor cosmetic/spacing polish or non-critical typing omissions; contract tests present and passing.
- **3 (Usable with Friction)**: Works, but noticeably cluttered, unrefined hierarchy, missing mobile trigger, loose types (`any`/untyped returns), or missing/incomplete `api.test.ts` contract tests.
- **2 (Rough)**: Inconsistent styling, poor contrast, high cognitive load, layout glitches, trapped navigation, or failing contract tests.
- **1 (Broken)**: Misaligned elements, unreadable text, broken responsive collapse, blank root route, or compiler type errors.

## Process

### Step 1: Capture Live Screenshots
1. Determine the target route (e.g. `http://localhost:5172/configuration/family` or `/configuration/family`).
2. Ensure the dev server is active via `./dev.sh dev`. If ports `:5171` or `:5172` are occupied, the maintainer is already running the server outside—do not attempt to terminate processes; proceed directly to capture screenshots against the active instance.
3. Run screenshot capture via `./dev.sh`:
   ```bash
   ./dev.sh ui capture <target-url-or-path>
   ```
   Captures into `.scratch/ui-review/`:
   - `desktop-dark.png`
   - `desktop-light.png`
   - `mobile-dark.png`
4. View the screenshots using `view_file` to evaluate visual fidelity and shell layout.

> **Completion Criterion**: Screenshots generated in `.scratch/ui-review/` and visually inspected.

### Step 2: Spawn Parallel Sub-Agents
Dispatch two sub-agents in parallel:

- **Sub-Agent 1 (Visual & Design System Critic)**:
  - Input: Screenshot paths and target `.svelte` files.
  - Task: "Audit the page against **Axis 1 (Aesthetics & Visual Hierarchy)** and **Axis 3 (Design System Fidelity)**. Evaluate eye flow, typography rhythm, OLED black/white contrast, token discipline (canonical Tailwind v4, zero raw hex), and unadorned labels. Cite exact file and line numbers for every finding. Score Axis 1 and 3 (1–5) with concise rationale. Max 350 words."

- **Sub-Agent 2 (UX, Shell Navigability, a11y & Type Rigidity Auditor)**:
  - Input: Screenshot paths and feature source files (`types.ts`, `store.svelte.ts`, `api.ts`, `api.test.ts`, components).
  - Task: "Audit the page against **Axis 2 (UX & Shell Navigability)**, **Axis 4 (Accessibility & Semantics)**, and **Axis 5 (Resilience, Type Rigidity & Contract Verification)**. Verify absence of patronizing instructional subtitles, check touch targets, ensure mobile hamburger trigger is accessible, audit `aria-label`s on icon buttons, enforce Rust-grade TypeScript (zero `any`, zero blind `as T` JSON casts, mandatory `schema: parseX` runtime decoders, strict `T | null`, tagged unions, explicit return types), and **verify that `frontend/src/lib/features/<feature>/api.test.ts` exists and passes with complete endpoint, envelope, and schema decoder coverage against `MemoryTransportAdapter`**. Cite exact file and line numbers. Score Axis 2, 4, and 5 (1–5) with concise rationale. Max 350 words."

> **Completion Criterion**: Both sub-agents return structured findings with 1–5 scores and line-numbered citations.

### Step 3: Aggregate Scorecard & Punchlist
Synthesize findings into two sections:
1. **5-Axis Scorecard**: Table listing 1–5 scores and a 1-sentence assessment per axis, plus the composite average.
2. **Segregated Remediation Punchlist**:
   - **Bucket A: Mechanical Polish (1-Click Auto-Fix)**: Deterministic items with file/line citations (missing `aria-label`, token fix, unadorned label rename, explicit return type, missing contract test).
   - **Bucket B: Design Decisions (Human Choice)**: Architectural or layout trade-offs phrased as plain-English choices with 2 concrete alternatives.

> **Completion Criterion**: Scorecard and segregated punchlist presented in chat.

### Step 4: Remediation Gate
Ask the human:
1. *"Would you like me to auto-apply the [N] mechanical fixes from Bucket A right now?"*
2. Present options for any Bucket B items.
Pause execution. **Do not write or edit code before the human responds.**

> **Completion Criterion**: Human explicitly approves Bucket A execution and provides decisions for Bucket B.

### Step 5: Patch & Verify
1. Apply approved fixes strictly within the feature directory and layout shell.
2. Run `./dev.sh ui flint`, `./dev.sh all check`, and `./dev.sh ui test feature <feature>` to verify zero test, lint, or type errors.
3. Re-run `./dev.sh ui capture <target-url-or-path>` and view updated screenshots to verify visual resolution.
4. If an audit finding exposes an elusive bug, state machine regression, or test flake, trigger **[`/diagnosing-bugs`](../diagnosing-bugs/SKILL.md)** to isolate the defect with a minimal reproduction before fixing.

> **Completion Criterion**: `./dev.sh all check` passes with 0 errors, `./dev.sh ui test feature <feature>` passes with 0 failures, and updated screenshots verify visual and navigation fixes.

## Next Flow

- **Phase 2 Implementation**: Once the UI is audited and frozen, hand off to Phase 2 backend TDD via **[`/tdd`](../tdd/SKILL.md)**.
- **Full PR Review**: When full-stack implementation is complete, trigger **[`/code-review`](../code-review/SKILL.md)** to run a two-axis (Standards + Spec) audit of the entire branch before merging.
