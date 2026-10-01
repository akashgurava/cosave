# Full-Stack Live API Integration Testing and Contract Drift Prevention

Frontend API clients and runtime schema decoders must be validated against a live, running Axum backend via automated live API integration tests (`*.integration.test.ts`) executed through `./dev.sh ui test:integration` and enforced by `./dev.sh all audit`, eliminating silent contract drift caused by isolated in-memory mock adapters.

## Context & Decision

In Phase 1 and Tier 2 frontend testing, `MemoryTransportAdapter` simulates network requests against in-memory fixture arrays. While this provides instantaneous test execution for UI prototypes and rune stores, mock adapters suffer from a structural vulnerability: **contract drift**. 

When backend route paths (e.g. migrating categories to `/api/v1/config/*`), response structures (nested trees vs flat lists), entity primary key types (64-bit integer IDs vs string IDs), or mutation semantics (Command-Query Separation / CQS) change in the backend, mock tests can remain 100% green while real-world application calls fail with 404 or contract decoding errors.

To permanently bridge this gap without compromising local developer velocity, we establish:

1. **Dedicated Live Integration Test Files (`*.integration.test.ts`)**:
   - Reside alongside feature modules (e.g., `frontend/src/lib/features/categories/categories.integration.test.ts`).
   - Use `FetchTransportAdapter` configured with `baseUrl` to issue real HTTP requests over the network.
   - Run in Node.js with built-in session cookie persistence to test protected authenticated routes.
   - Execute against the real SQLite database schema, validating that actual SQLite database seeds decode cleanly through runtime `parseX` decoders and that mutation endpoints respect CQS contracts.

2. **Automated Ephemeral Backend Test Harness**:
   - The CLI workflow `./dev.sh ui test:integration` (and `./dev.sh all test:integration`) detects if an active backend is already listening on port `:5171`.
   - If no backend is active, the harness automatically spawns an ephemeral Axum backend on port `:5199` using an isolated temporary SQLite database, awaits server health, runs Vitest against `:5199`, and cleanly terminates the child process on exit via trap handlers.

3. **Mandatory Audit Gate**:
   - `./dev.sh all audit` runs `test:integration` immediately after unit tests. A full workspace audit will not pass if frontend decoders diverge from live Axum responses.

## Consequences

- Silent contract drift between Axum routes and SvelteKit API adapters is completely prevented.
- Discrepancies between default SQLite database seeds and frontend presentation assumptions are caught automatically.
- Everyday unit and rune store tests remain headless, fast, and in-memory, while full-stack integration tests provide high-confidence verification on demand and before merging.
