---
name: ui-audit
description: Rigorous type-system, anti-looseness, and relational-indexing audit for frontend code. Scans for blind casts (as), nullish fallbacks (??), optionality (?.), loose brand unions, non-null assertions (!), and ad-hoc .find scans. Triggers on auditing frontend types, verifying strictness, eliminating type coercion, or reviewing folder by folder.
---

# UI Type Rigidity & Anti-Looseness Audit

Audit frontend code against CoSave's strict engineering standards, ADR 0006 (Relational Indexing), and ADR 0007 (Anti-Looseness & Explicit Branching).

The Rust backend is our authoritative Single Source of Truth; the frontend is a zero-looseness mirror. This skill runs the disclosed Python engine (`.agents/skills/ui-audit/scripts/ui_audit_types.py`) to extract deep line-level diagnostics, rummages through the verbose finding stream, and translates raw AST/regex matches into an authoritative scorecard and remediation punchlist.

---

## The 4 Audit Pillars

1. **Nominal Branding & Smart Constructors**
   - **Zero Blind Assertions**: `as Brand` (`as CategoryId`, `as MinorUnits`) is forbidden outside smart constructors. Primitives must flow strictly through runtime decoders or companion constructors: `*Id(raw)` and `MinorUnits(raw)`.
   - **Zero Loose Unions**: Signatures like `id: CategoryId | number` or `id: CategoryId | string` are banned. Enforce strict single-brand types.
   - **Zero Legacy Converters**: Ensure `to*Id(...)` and `toMinorUnits(...)` remain completely eradicated.

2. **Zero Defensive Liquidations & Fallbacks**
   - **Zero Nullish Coalescing (`??`) on Domain Data**: Never liquidate missing domain state or database entities with default placeholders (`name ?? "Unknown"` or `cat ?? { id: 1 }`). Branch explicitly or assert invariant presence via `expectPresent(val, action, message)`.
   - **Optional Arguments Bags vs Domain Liquidation**: Distinguish between domain entity fallback liquidation (strictly banned) and standard optional configuration bag defaults (`options?.duration ?? 5000` or `opts?.title ?? defaultTitle`). Do NOT turn standard optional configuration defaults into unreadable nested ternaries.
   - **Zero Logical OR Liquidators (`||`)**: Banned for defaulting values (`title || defaultTitle`).
   - **Zero Optional Chaining (`?.`) on Invariants**: Chaining on schema-guaranteed relationships hides backend or store synchronization bugs.

3. **Compiler Invariants, Strict Branching & Eradication of `undefined`**
   - **Zero `any`**: Strictly banned crate-wide. Replace with `unknown` paired with type decoders.
   - **Zero Non-Null Assertions (`!`)**: Banned across all product and test code.
   - **Zero `undefined` in Domain Contracts ("Always Provide the Box")**:
     - `undefined` represents accidental omission or uninitialized engine state.
     - Never use `T | undefined` or optional modifiers (`prop?: T`) on domain models, DTOs, or store contracts.
     - Always provide the property explicitly; if a value can be intentionally empty, type it strictly as `T | null` (`action: string | null`, not `action?: string`).
   - **Strict Equality Only**: `==` and `!=` are banned. Explicit `===` and `!==` only.
   - **Zero Implicit Truthy/Falsy Coercion**: `if (!x)` or `if (x)` is banned. Always compare explicitly: `val === true`, `val === false`, `count > 0`, `str.length > 0`, `val === null || val === undefined`.

4. **Relational Indexing & Invariant Lookups (ADR 0006)**
   - **Zero Ad-Hoc Array Scans**: `.find(...)` inside component markup, table rows, and store getters is banned.
   - **Constant Time SvelteMap Lookups**: Entities must be indexed in stores via `$derived(new SvelteMap(...))` and resolved through fail-fast invariant accessors (`getCategory(id)`).

---

## Audit Workflow

```
Target Selection → Engine Execution → Triage & Rummage → Prioritized Punchlist
```

### Step 1: Target Selection & Scope

Determine the target path and scope:
- **Full Feature Folder**: `frontend/src/lib/features/<feature>` (e.g. `categories`, `family`, `transactions`).
- **Top-Level Root Only**: `frontend/src/lib --top-level-only` (ignores subfolders like `features/` or `api/`).
- **Targeted File**: `frontend/src/lib/features/<feature>/store.svelte.ts`.

### Step 2: Run the Disclosed Engine

Run the audit engine with `uv` in JSON or verbose stream mode:

```bash
# 1. Full directory audit (verbose stream with surrounding context & summary)
uv run .agents/skills/ui-audit/scripts/ui_audit_types.py <path>

# 2. Top-level only (no recursion into subdirectories)
uv run .agents/skills/ui-audit/scripts/ui_audit_types.py <path> --top-level-only

# 3. Exclude test files to focus on production runtime code
uv run .agents/skills/ui-audit/scripts/ui_audit_types.py <path> --exclude-tests

# 4. JSON stream for programmatic deep triage
uv run .agents/skills/ui-audit/scripts/ui_audit_types.py <path> --json
```

> **Completion Criterion**: The engine runs cleanly to completion and outputs findings with file paths, line numbers, column offsets, and context.

### Step 3: Rummage & Triage Findings

The engine outputs a raw, verbose diagnostic stream. Do not dump the engine's raw output directly to the user. As the auditor agent, you must rummage through the findings and apply domain judgement:

1. **Verify Context**: Inspect the actual source code surrounding each match.
   - Separate legitimate boolean validation checks (e.g. `typeof x !== "string" || x.length === 0`) from illegal fallback liquidators (`opts.title || defaultTitle`).
   - Separate vendor shims (e.g. official shadcn utility types in `utils.ts`) from product code violations.
   - Separate valid internal companion constructor return sites (`return raw as Brand;` inside `types.ts`) from call-site blind casts.
   - Separate standard optional configuration bag defaults (e.g. `options?.duration ?? 5000`) from domain state liquidation.

2. **The Anti-Overaction Principle (Simplest Honest Fix)**:
   - **Delete Redundant Boilerplate Instead of Synthesizing Verbose Checks**: If a property is already typed `T | undefined` on a known domain object (e.g. `err.action ?? undefined`), the fix is NOT a clumsy double-null check `err.action !== null && err.action !== undefined ? err.action : undefined`. The honest, zero-overhead remedy is simply `err.action`.
   - **No Ceremony on Optional Config Bags**: When a function takes optional options (`options?: ToastErrorOptions`), fallback defaults (`options?.title ?? defaultTitle`) are idiomatic and safe. Do NOT prescribe verbose ternaries.
   - **Trust Validated Models**: Do not wrap trusted domain properties in defensive wrappers. Liquidating defaults on entities should be removed or asserted via `expectPresent`.
   - **No Compromises on True Violations**: Never compromise on eliminating blind `as Brand` casts, `any`, `!`, or ad-hoc `.find()` scans. Keep remedies minimal, idiomatic, and clean.

3. **Classify Severity**:
   - `[FATAL]`: Blind brand casts (`as Brand`), non-null assertions (`!`), `any`, loose brand unions, loose equality (`==`).
   - `[DEFENSIVE]`: Nullish coalescing fallbacks (`??`), logical OR fallbacks (`||`), optional chaining on invariant models (`?.`), accidental optional modifiers (`prop?: T`) on domain models, and `undefined` type annotations (`| undefined`).
   - `[STRICTNESS]`: Implicit falsy/truthy checks (`if (!x)`), ad-hoc `.find()` scans in templates.
   - `[EXEMPT]`: Validated constructor internals, shadcn upstream shims, boolean validation conditions, optional configuration bag defaults (e.g. `options?: ToastOptions`).

### Step 4: Persist the Audit Report (.audit/ Markdown File)

Before responding in chat/terminal, write the complete audit report to a timestamped Markdown report:

```
.audit/ui-audit-<target-slug>-YYYYMMDD-HHMMSS.md
```

*(e.g., `.audit/ui-audit-frontend-src-lib-top-level-20261011-175500.md` or `.audit/ui-audit-categories-20261011-175500.md`)*

#### Markdown File Linking & Content Structure Rules (For `.md` Files Viewed in VS Code Markdown Preview & GitHub)
In VS Code's Markdown Preview (`markdown-language-features`), absolute `file://` links are blocked for security. The authoritative format that navigates directly to code and line numbers is **relative paths from the report file with `#L<line>` anchors**:
- **File Headers**: `### 📄 [frontend/src/lib/toast.ts](../frontend/src/lib/toast.ts)`
- **Finding Citations**: `1. [frontend/src/lib/toast.ts#L21](../frontend/src/lib/toast.ts#L21) [DEFENSIVE] [FALLBACK_NULLISH] ...`
- **Exempt Citations**: `• [frontend/src/lib/types.ts#L97](../frontend/src/lib/types.ts#L97)`

*(Since the report lives in `.audit/`, paths beginning with `../frontend/...` resolve directly to workspace files and jump to the exact line number on click in VS Code Markdown Preview).*

**Exemption Reporting Contract in `.audit/*.md`**:
Exemptions in the report must never be condensed into brief summary bullets with buried comma-separated links. Every report must follow the expanded format with dedicated subheadings (`### A. ...`, `### B. ...`), 3–5 individual bullet citations showing the exact code snippet, and an explicit **Verdict**. This ensures structural parity and predictable reports across audit runs.

### Step 5: Deliver the Terminal / Chat Response

In the chat response and CLI terminal output, the linking requirements are completely different from a `.md` file.

#### Terminal Plain-Text Linking Rules (For VS Code Integrated Terminal)
VS Code's terminal regex parser (`TerminalLocalLinkDetector`) does NOT parse Markdown link tags `[text](url)`. Instead, it scans raw text for plain file paths with colon-separated coordinates:
1. **Plain Path Pattern**: Must follow `path/to/file.ext:line:col` with NO wrapping characters (no brackets `[]`, parentheses `()`, or backticks `` ` `` touching the path or line/col numbers).
2. **Path Anchoring**: Must be workspace-relative (`frontend/src/lib/...`) or absolute.

Format the chat/terminal response as:

#### Section 1: Executive Scorecard
Render a Markdown table summarizing the findings:

| Metric | Count |
|---|---|
| Files Scanned | `<N>` |
| Files with Findings | `<N>` |
| Total Findings | `<N>` |
| Actionable Fatal | `<N>` |
| Actionable Defensive | `<N>` |
| Actionable Strictness | `<N>` |
| Exempt / False Positives | `<N>` |

#### Section 2: Prioritized Remediation Punchlist
Organize actionable findings by file, ranked by severity.

- **File Header**:
  ```markdown
  ### 📄 frontend/src/lib/toast.ts
  ```
- **Finding Citation**:
  ```markdown
  1. frontend/src/lib/toast.ts:21:23 [DEFENSIVE] [FALLBACK_NULLISH] Nullish coalescing default (??)
  ```
  *(Notice: `frontend/src/lib/toast.ts:21:23` is bare text so VS Code terminal underlines it on hover and Cmd+Click opens line 21, column 23 directly).*

For each item, specify:
- **Location**: Standalone `path:line:col` string for terminal clickability.
- **Severity & Rule**: e.g. `[FATAL] [BLIND_BRAND_CAST]` or `[DEFENSIVE] [FALLBACK_NULLISH]`.
- **Offending Code**: The exact matching code snippet.
- **Rationale**: Why this violates Rust-grade strictness or ADR invariants.
- **Concrete Remedy**: The simplest, honest replacement code (e.g. companion constructors for brands, deleting redundant defensive fallbacks rather than synthesizing bloated ternaries).

#### Section 3: Exempt Findings & Invariant Analysis
List findings examined during triage that were determined to be valid or exempt (e.g., smart constructor implementations, boolean validation guards, vendor types, low-level boundary guards).

**Zero Collapsing / Predictable Structure Invariant**:
- Never collapse exempt groups into a single vague summary line (e.g. `Network Boundary Nullity Checks (file#L1, file#L2): Low level...`).
- Always group into clear categorical subsections (e.g. `### A. HTTP Request Options Bags`, `### B. Network Boundary Nullity Guards`).
- For each category, provide **expanded citations with concrete code expressions** for 3–5 representative matches (or all matches if fewer than 5):
  ```markdown
  ### C. Cookie & Boundary Nullity Guards (Exempt)
  - [frontend/src/lib/api/transport.ts#L47](../frontend/src/lib/api/transport.ts#L47): `headers["cookie"] === undefined`.
  - [frontend/src/lib/api/transport.ts#L65](../frontend/src/lib/api/transport.ts#L65): `setCookie !== null`.
  - [frontend/src/lib/api/contracts.ts#L101](../frontend/src/lib/api/contracts.ts#L101): `value !== null` in `isObject` type predicate.
  - [frontend/src/lib/api/contracts.ts#L105](../frontend/src/lib/api/contracts.ts#L105): `raw === null || raw === undefined` in `parseNull`.
  - **Verdict**: **Exempt**. Standard low-level defensive guards at the boundary between untyped network fetch responses and JavaScript runtime.
  ```
- Structure and detail must remain predictable and identical between audit runs.

#### Section 4: Next Action Gate
Conclude with a clear, single-question prompt asking the user for authorization to apply the fixes or proceed to the next folder.
