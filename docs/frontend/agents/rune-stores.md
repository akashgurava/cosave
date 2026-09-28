# Svelte 5 Rune Stores & State Architecture

Guidelines for managing reactive state, asynchronous flows, and component encapsulation in Svelte 5.

## Trigger & Scope

Consult this guide whenever authoring feature state, handling async data fetches, creating Svelte 5 runes, or building UI stores.

## 1. Class-Based Rune Stores (`.svelte.ts`)

Mirroring Rust's struct encapsulation (`struct` with private fields and `impl` methods), reactive state is encapsulated in domain store classes defined in `.svelte.ts` files.

### Rules:
1. **Private State**: State variables must use native private fields (`#state = $state(...)`). Never expose mutable `$state` fields directly to consumers.
2. **Read-Only Getters**: Expose reactive state through read-only accessors (`get state()`, `get items()`).
3. **Explicit Action Methods**: Mutations must occur through named methods (`async load()`, `async deleteItem(id)`).
4. **No Direct External Mutation**: Consumers cannot execute `store.items.push(...)` or `store.loading = false`.

### Implementation Pattern:
```ts
// frontend/src/lib/features/categories/category-store.svelte.ts
import type { AsyncState } from "$lib/types/core";
import type { CategoryItem, CategoryTransport } from "./types";
import { ApiError } from "$lib/api";

export class CategoryStore {
  #state = $state<AsyncState<readonly CategoryItem[]>>({ status: "idle" });
  #transport: CategoryTransport;

  constructor(transport: CategoryTransport) {
    this.#transport = transport;
  }

  get state(): AsyncState<readonly CategoryItem[]> {
    return this.#state;
  }

  async load(): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const data = await this.#transport.fetchHierarchy();
      this.#state = { status: "success", data: Object.freeze(data.categories) };
    } catch (err) {
      const action = err instanceof ApiError ? (err.action ?? "CATEGORIES.LOAD.FAILED") : "CATEGORIES.LOAD.FAILED";
      const message = err instanceof Error ? err.message : "Failed to load categories";
      this.#state = { status: "error", error: { action, message } };
    }
  }
}
```

## 2. Impossible States Are Unrepresentable

Always model asynchronous query state as a tagged discriminated union:

```ts
export type AsyncState<T, E = ErrorPayload> =
  | { readonly status: "idle" }
  | { readonly status: "loading" }
  | { readonly status: "success"; readonly data: T }
  | { readonly status: "error"; readonly error: E };
```

Never manage separate flags like `let loading = false; let error = null; let data = null;`. A tagged union guarantees you can never display stale data alongside an active error spinner.

## 3. Two-Phase Decoupled Lifecycle

1. **Phase 1 (UI First)**:
   * Instantiate store with `MemoryTransportAdapter(mockData)`.
   * Test interactions, edge cases, and loading states without any running backend.
   * Freeze UI prototypes.
2. **Phase 2 (Backend Wire-Up)**:
   * Instantiate store with `HttpTransportAdapter()`.
   * Connect to live Axum endpoints with zero modifications to component view code.
