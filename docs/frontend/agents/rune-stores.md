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
import type { CategoryHierarchyResponse, CategoryTransport } from "./types";
import { ApiError } from "$lib/api";

export class CategoryStore {
  #state = $state<AsyncState<CategoryHierarchyResponse>>({ status: "idle" });
  #transport: CategoryTransport;

  constructor(transport: CategoryTransport) {
    this.#transport = transport;
  }

  get state(): AsyncState<CategoryHierarchyResponse> {
    return this.#state;
  }

  async load(): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const data = await this.#transport.fetchHierarchy();
      this.#state = { status: "success", data: Object.freeze(data) };
    } catch (err) {
      const action =
        err instanceof ApiError && err.action !== null && err.action !== undefined
          ? err.action
          : "CONFIG.CATEGORIES.FETCH_HIERARCHY.FAILED";
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

## 4. Reactive `$derived` Map Indexing & Invariant Getters

Relational lookups (joining items to categories, accounts, or members) must execute in $O(1)$ constant time through reactive store indices. Never require views to execute ad-hoc `Array.prototype.find()` searches.

### Rules:
1. **Reactive Map Indices**: Stores derive reactive `Map` lookups via `$derived`:
   ```ts
   export class CategoryStore {
     #categories = $state<readonly Category[]>([]);
     #categoryMap = $derived(new Map(this.#categories.map((c) => [c.id, c])));

     /**
      * Authoritative O(1) lookup. Asserts existence and returns non-nullable entity.
      */
     getCategory(id: CategoryId): Category {
       const cat = this.#categoryMap.get(id);
       if (cat === undefined) {
         throw new InvariantViolationError("STORE.GET_CATEGORY.NOT_FOUND", `Category ${id} not found in store`);
       }
       return cat;
     }
   }
   ```
2. **Fail-Fast Invariant Getters**: Foreign key relationships are structural domain invariants. Store getters must return guaranteed non-nullable entities (`getCategory(id) -> Category`). Missing IDs must throw `InvariantViolationError` immediately to catch data and preload defects at the source.
3. **Zero In-View `.find()` Scans**: Prohibit calling `.find()` inside Svelte component templates, table rows, and sort comparators. Read directly from store getters or pass pre-joined view models.
4. **Zero Bogus Fallback Synthesis**: Never synthesize fake fallback objects (e.g. `find(...) ?? { id: 1, name: "USD" }`) or mask missing lookups with `?? "Custom"`. If an association is optional in the domain, model it explicitly as `T | null`.

## 5. Fail-Fast Action Methods (Throw, Never Return False)

Store mutating and action methods must throw typed `ApiError` or domain errors on failure:
- **Never Return Boolean `false`**: Methods that silently return `false` on failure prevent callers from distinguishing failure causes, reading action tokens, or displaying error badges.
- **Infallible on Success, Throwing on Failure**: Action methods return the newly created/updated entity or void on success, and throw on failure.

## 6. Svelte 5 Component Template Invariants

- **`{@const}` Placement**: `{@const}` tags must be immediate children of template blocks (`{#if}`, `{#each}`, `{#snippet}`) or components, never arbitrary HTML tags (`<span>`, `<div>`). Multi-step view mappings must be declared in `<script>` via `$derived.by(...)`.
- **Explicit Boolean Bindings**: Modal and dialog bindings must evaluate boolean state explicitly (`open === true`, `isOpen === false`).

