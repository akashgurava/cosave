# Svelte 5 Class-Based Rune Stores and Discriminated Union Async State

Frontend reactive state is encapsulated in domain classes in `.svelte.ts` files with `#private` fields and explicit mutation methods, while asynchronous operations model their lifecycles as strict discriminated unions (`AsyncState<T>`).

## Context & Decision

Frontend state often suffers from two anti-patterns:
1. **Unencapsulated State**: Scattering mutable state variables across component files or exporting raw writable objects, allowing any consumer to mutate state arbitrarily.
2. **Boolean Flag Soup**: Managing async operations with multiple independent flags (`isLoading`, `isError`, `data`, `error`), creating impossible states (e.g. `isLoading: true` while `data` is populated, or `isError: true` with no error message).

To enforce Rust-like encapsulation and state-machine rigor:

1. **Class-Based Rune Stores (`.svelte.ts`)**:
   - Feature state is encapsulated within classes inside `.svelte.ts` files (where Svelte 5 universal reactivity runes are compiled).
   - Internal reactive variables use native JavaScript private fields: `#state = $state(...)`.
   - Outside consumers read data exclusively through read-only getters (`get items(): readonly Item[]`).
   - Mutations are strictly restricted to explicit action methods (`async load()`, `createItem(...)`), preventing arbitrary external tampering (mirroring Rust's private struct fields and `impl` methods).

2. **Discriminated Union Async State (`AsyncState<T>`)**:
   - Asynchronous operations are modeled as a rigid tagged union:
     ```ts
     export type AsyncState<T, E = ErrorPayload> =
       | { readonly status: "idle" }
       | { readonly status: "loading" }
       | { readonly status: "success"; readonly data: T }
       | { readonly status: "error"; readonly error: E };
     ```
   - Illegal states are physically unrepresentable at compile time.
   - Svelte templates use exhaustive checks against `status` (`"idle"`, `"loading"`, `"success"`, `"error"`).

3. **Decoupled Two-Phase Lifecycle Integration**:
   - In **Phase 1 (UI First)**, rune stores consume a `TransportAdapter` backed by `MemoryTransportAdapter` and `mock.ts`. The UI is developed, interactively switched, and frozen without waiting for backend implementation.
   - In **Phase 2 (Backend Wire-Up)**, the store swaps the transport to `HttpTransportAdapter` connecting to Axum endpoints with zero modifications to the UI components.

## Consequences

- Components cannot illegally mutate state from the outside.
- Eliminates impossible visual states (such as loading spinners remaining active after errors).
- UI development proceeds completely decoupled from backend readiness.
