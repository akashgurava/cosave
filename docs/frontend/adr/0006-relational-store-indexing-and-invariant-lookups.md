# Relational Store Indexing, Invariant Assertions, and Zero Type Liquidation

Frontend domain stores maintain reactive `$derived` `Map` indices for relational entities ($O(1)$) and expose invariant-asserting getters (`getCategory(id) -> Category`) that fail fast. Manual `Array.prototype.find()` scans in views and synthetic fallback objects are strictly forbidden.

## Context & Decision

When presenting relational data (such as transactions referencing categories, accounts, and members), frontend code often degenerates into three anti-patterns:
1. **Manual Frontend JOINs**: Views and row components execute ad-hoc linear scans (`categories.find(c => c.id === tx.categoryId)`), degrading performance to $O(N \times M)$ across lists and sort comparators.
2. **Defensive Type Liquidation**: Because `Array.prototype.find()` returns `T | undefined`, guaranteed database relations are degraded to optional slop. Templates and view components litter markup with optional chaining (`?.`), nullish coalescing (`?? null`), and empty string fallbacks (`?? ""`).
3. **Bogus Fallback Synthesis**: When lookups fail, components synthesize fake fallback objects (e.g. `find(...) ?? { id: 1, name: "USD" }` or `find(...) ?? { name: "Expense" }`), silently masking foreign key corruptions, uninitialized store states, and broken joins.
4. **Nominal Brand Erasure**: Developers cast raw values using blind `as Brand` assertions (e.g. `id as CategoryId`, `val as MinorUnits`) in click handlers and modal forms to silence compiler checks instead of using validated constructors or typed store models.

To enforce relational integrity and Rust-grade type rigor on the presentation layer, we enforce:

1. **Reactive `$derived` Map Indexing in Stores**:
   - Feature stores store raw collections as private state (`#categories`, `#members`, `#accounts`).
   - Stores synchronously maintain reactive indexed maps using Svelte 5's `$derived`:
     ```ts
     #categoryMap = $derived(new Map(this.#categories.map((c) => [c.id, c])));
     #memberMap = $derived(new Map(this.#members.map((m) => [m.id, m])));
     ```
   - All relational resolutions execute in $O(1)$ constant time.

2. **Invariant-Asserting Getters ("Fail Fast, Never Liquidate")**:
   - Stores expose non-nullable entity accessors:
     ```ts
     getCategory(id: CategoryId): Category {
       const cat = this.#categoryMap.get(id);
       if (!cat) {
         throw new Error(`[InvariantViolation] Category ${id} not found in store`);
       }
       return cat;
     }
     ```
   - In development and prototype modes, missing foreign keys throw immediately with an explicit error, pinpointing broken test fixtures or missing preloads.
   - In production environments, store error boundaries or dedicated tombstone resolvers (e.g. `getCategoryOrTombstone(id)`) handle soft-deleted/archived entities with typed domain semantics.

3. **Zero `Array.prototype.find()` in Views & Templates**:
   - Calling `.find()` inside templates, row components, or sort comparators is prohibited.
   - Views consume either:
     - Direct $O(1)$ store getters (`categoryStore.getCategory(tx.categoryId).name`), or
     - Denormalized / enriched view models prepared by the store before rendering.

4. **Zero Synthesized Fallback Objects**:
   - Fabricating dummy domain entities via `?? { ... }` or defaulting missing labels to `?? "Custom"` to pacify nullability is banned.
   - Foreign key integrity is an invariant. If the entity is required, it must exist. If an association is truly optional in the domain, it must be explicitly typed as `T | null` at the schema level.

5. **Nominal Branding Integrity**:
   - Blind type casting via `as Brand` in presentation components, click handlers, and forms is prohibited.
   - Identifiers must retain their branded type through validated constructors (e.g. `toMinorUnits(raw)`) or typed selection models.

## Consequences

- Templates and view components remain clean, non-nullable, and free of defensive `?.` and `??` clutter.
- Relational lookups run in $O(1)$ without UI hitching during sorting or high-frequency list rendering.
- Foreign key mismatches and uninitialized store dependencies fail immediately at the source rather than silently corrupting user data.
- Domain models and store contracts remain strictly aligned with the backend relational schema.
