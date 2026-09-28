# Frontend 3-Tier Testing Standards

Guidelines for authoring frontend unit, contract, and end-to-end tests without vanity tests or brittle snapshot regressions.

## Trigger & Scope

Consult this guide whenever authoring frontend tests (`*.test.ts`), verifying API contract compliance, or running Playwright tests.

## 1. The 3-Tier Testing Pyramid

Mirroring the backend test pyramid, frontend tests are structured into three distinct tiers:

```
        ▲
       / \     Tier 3: User Journey Integration (Playwright)
      /   \    Real browser, user actions, semantic locators
     /-----\
    /       \    Tier 2: Rune Store & Contract Tests (Vitest)
   /         \   State machine transitions & MemoryTransportAdapter
  /           \  Verifies API contract before backend exists
 /-------------\
/               \  Tier 1: Pure Domain & Decoder Unit Tests (Vitest)
─────────────────  Zero DOM, microsecond speed, tests parseX & math
```

### Tier 1: Pure Domain & Decoder Unit Tests (Vitest)
* **Location**: `frontend/src/lib/**/*.test.ts` (e.g. `types.test.ts`, `currency.test.ts`).
* **Requirements**:
  * Asserts `parseX(raw: unknown)` narrows valid objects and freezes collections.
  * Asserts invalid shapes throw `ContractViolationError`.
  * Tests nominal constructors (`toAmountCents(raw)`).
  * Executes with zero DOM overhead in microseconds.

### Tier 2: Rune Store & Contract Tests (Vitest)
* **Location**: `frontend/src/lib/features/<feature>/api.test.ts`, `store.test.ts`.
* **Requirements**:
  * Verifies state machine transitions of `.svelte.ts` rune stores (`idle` -> `loading` -> `success` / `error`).
  * Verifies `MemoryTransportAdapter` satisfies the contract, throws typed `ApiError`, and matches backend action tokens.
  * Ensures Phase 1 mock implementations are 100% compliant with the agreed wire schema before Phase 2 begins.

### Tier 3: Black-Box User Journey Tests (Playwright)
* **Location**: `frontend/e2e/`.
* **Requirements**:
  * Tests multi-step user workflows (login, creating a category, updating theme).
  * Queries elements via accessible roles (`getByRole("button", { name: "Save" })`) and user-visible text.
  * Never targets internal CSS selectors or fragile class chains.

## 2. Banned Testing Anti-Patterns

1. **Zero Shallow Snapshots**: Never use `toMatchSnapshot()` on HTML trees or component renders. They fail on cosmetic class updates without testing behavior.
2. **Zero Internal Component Mocking**: Do not mock internal child components or store guts. Test through public interfaces.
3. **Zero Blind Casts in Tests**: Never use `as any` or blind type assertions to construct test payloads; use valid domain fixtures, builder helpers, or `@total-typescript/shoehorn` (`import { partial } from "@total-typescript/shoehorn"`).

## 3. Test Type Narrowing under `noUncheckedIndexedAccess`

Under `noUncheckedIndexedAccess: true`, accessing array indices (`items[0]`) yields `T | undefined`. 

**The Trap**: Vitest's runtime assertion `expect(items[0]).toBeDefined()` **does not narrow** TypeScript's type.

**The Solution**:
- Use optional chaining for assertions:
  ```ts
  expect(items[0]?.id).toBe("expected-id");
  ```
- Or use an explicit guard before accessing multiple properties:
  ```ts
  const first = items[0];
  expect(first).toBeDefined();
  if (first === undefined) return; // Narrows 'first' to T
  expect(first.name).toBe("Sample");
  ```
