# 3-Tier Frontend Testing Pyramid and Zero Vanity Tests

The frontend enforces a strict 3-tier testing pyramid: (1) Pure Domain and Decoder Unit Tests, (2) Rune Store and Contract Tests, and (3) Black-Box User Journey Tests, explicitly banning brittle vanity snapshots.

## Context & Decision

Frontend testing is frequently compromised by high-maintenance, low-value vanity tests—specifically shallow DOM component snapshots that fail on minor CSS or markup changes without verifying meaningful behavior.

Mirroring the backend's 3-tier testing architecture, we establish:

1. **Tier 1: Pure Domain & Runtime Decoder Unit Tests (Vitest)**:
   - **Target**: `types.test.ts`, `domain.test.ts`, and financial utility files.
   - **Characteristics**: Fast, zero DOM, microsecond execution.
   - **Scope**: Asserts that `parseX(raw)` decodes valid payloads and rejects corrupted payloads with `ContractViolationError`; tests branded type constructors; tests financial math and date calculations.

2. **Tier 2: Rune Store & Contract Tests (Vitest)**:
   - **Target**: `store.test.ts` and `api.test.ts`.
   - **Characteristics**: Fast, headless, state-machine focused.
   - **Scope**: Asserts `.svelte.ts` rune stores transition correctly across the `AsyncState` cycle (`idle` -> `loading` -> `success` / `error`); tests `MemoryTransportAdapter` ensuring mock implementations strictly enforce request validation and return structured error envelopes matching backend screaming action tokens. (Note: To prevent mock drift, full-stack live contract verification against the Axum backend is governed by ADR-0005 via `*.integration.test.ts`).

3. **Tier 3: Black-Box User Journey Tests (Playwright)**:
   - **Target**: `frontend/e2e/`.
   - **Characteristics**: Full browser environment testing real user interactions.
   - **Scope**: Verifies critical multi-step user workflows (login, category creation, family settings). Asserts UI elements strictly via accessible roles (`role="button"`) and user-facing text, never CSS selectors.

4. **Banned Practices**:
   - Shallow component snapshot tests (`toMatchSnapshot()`) are strictly forbidden.
   - Mocking internal component dependencies or implementation details is forbidden.

## Consequences

- Tests run reliably without brittle false-positives caused by styling changes.
- Tier 1 and Tier 2 tests provide instantaneous feedback during development.
- Contract tests serve as an executable specification linking Phase 1 (UI) and Phase 2 (Backend).
