# Zero-Tolerance Anti-Looseness, Explicit Branching, and Fail-Fast Contracts

The frontend enforces zero tolerance for loose JavaScript/TypeScript idioms (`??`, `?.`, `!`, implicit truthy/falsy coercion, and boolean-returning error suppression). All conditionals, domain validations, component bindings, and store mutations must use explicit equality predicates and throw typed errors on failure.

## Context & Decision

JavaScript's loose type coercion and TypeScript's permissive operators frequently mask critical bugs:
1. **Implicit Truthiness & Falsiness**: Evaluating `if (value)` or `if (!value)` silently misclassifies valid financial integers (`0` minor units treated as false), empty strings, or boolean flags (`open === false`).
2. **Defensive Nullish Coalescing (`??`)**: Defaulting missing values via `val ?? ""` or `opts ?? {}` acts as default liquidation. It silences uninitialized states, missing configuration, and broken API contracts.
3. **Unvalidated Optional Chaining (`?.`)**: Deep property chaining (`user?.profile?.settings?.theme`) degrades guaranteed schema relationships into speculative access, silently producing `undefined` when domain invariants are violated.
4. **Non-Null Assertions (`!`)**: The `!` assertion operator bypasses compiler safety without performing any runtime validation, creating blind spots for runtime null pointer exceptions.
5. **Silent Error Suppression in Store Actions**: Action methods that catch network/validation errors and return `false` instead of throwing prevent calling UI components from surfacing targeted error notifications and retry dialogs.
6. **Invalid Svelte 5 Template Const Placement**: Placing `{@const}` inside standard HTML elements (e.g. `<span>{@const x = ...}</span>`) violates Svelte 5 template constraints and breaks component builds.

To establish Rust-grade guarantees across the entire presentation layer, we enforce:

### 1. Strict Explicit Equality Predicates (No Implicit Truthy/Falsy Coercion)
All conditionals, guards, and loop filters must explicitly state the exact comparison:
- **Booleans**: Compare strictly against boolean literals:
  ```ts
  // FORBIDDEN
  if (isOpen) { ... }
  if (!isOpen) { ... }

  // REQUIRED
  if (isOpen === true) { ... }
  if (isOpen === false) { ... }
  ```
- **Numbers & Financial Quantities**: Validate scale and exact value:
  ```ts
  // FORBIDDEN
  if (count) { ... }
  if (!amount) { ... }

  // REQUIRED
  if (count > 0) { ... }
  if (count !== 0) { ... }
  if (Number.isInteger(amount) === false) { ... }
  ```
- **Strings**: Check explicit length or empty string:
  ```ts
  // FORBIDDEN
  if (query) { ... }
  if (!name) { ... }

  // REQUIRED
  if (query.length > 0) { ... }
  if (name.trim() === "") { ... }
  ```
- **Nullability & Presence**: Guard against both `null` and `undefined`:
  ```ts
  // FORBIDDEN
  if (item) { ... }
  if (!item) { ... }

  // REQUIRED
  if (item !== null && item !== undefined) { ... }
  if (item === null || item === undefined) { ... }
  ```

### 2. Prohibition of Loose Operators (`??`, `?.`, `!`) & Invariant Assertions (`expectPresent`)
- **Zero Loose `??`**: Do not use `??` to provide silent fallback defaults for domain data or configuration. Use explicit ternary branching or store invariant getters (`expectPresent`).
- **Zero Unchecked `?.`**: Do not use optional chaining to paper over domain models. If a relationship is guaranteed by the schema, access properties directly (`account.ownerMemberId`). If an association is optional, branch explicitly.
- **Zero Non-Null Assertions (`!`)**: The non-null assertion operator `!` is strictly forbidden across product and test code. Narrow types using runtime guards (`if (x === undefined) return;`) or `expectPresent(val, action, message)`.
- **Authoritative Invariant Assertions (`expectPresent`)**: `expectPresent` asserts that a value is guaranteed to exist by system design, business rules, or database relations. It is NOT a default fallback mechanism. When we call `expectPresent`, we expect the value to be present; if it is null or undefined, the application is in an illegal state and it immediately raises an `InvariantViolationError`. Never use `expectPresent` as an inline fallback inside ternary operations for optional parameters.

### 3. Fail-Fast Store Action Methods (Throw, Never Return False)
Store action methods must throw typed `ApiError` or domain errors on failure:
```ts
// FORBIDDEN: Silent failure returning boolean
async updateItem(id: ItemId, name: string): Promise<boolean> {
  try {
    await this.#transport.update(id, name);
    return true;
  } catch {
    return false; // Silently swallows error; UI cannot inspect action token
  }
}

// REQUIRED: Infallible on success, throws typed error on failure
async updateItem(id: ItemId, name: string): Promise<Item> {
  const updated = await this.#transport.update(id, name);
  this.#updateLocalState(updated);
  return updated; // Throws ApiError on network/validation failure
}
```

### 4. Svelte 5 Component Invariants & Script Derivations
- **`{@const}` Placement**: In Svelte 5, `{@const}` must be an immediate child of a template block (`{#if}`, `{#each}`, `{#snippet}`) or component, never an arbitrary HTML tag (`<span>`, `<div>`). Complex view transformations must be extracted into `<script>` using `$derived.by(() => { ... })`.
- **Component Prop Bindings**: Dialog and modal open bindings must evaluate boolean state explicitly (`open === true`, `onOpenChange={(isOpen) => { if (isOpen === false) onClose(); }}`).

## Consequences

- Completely eliminates silent false-positive and false-negative branches in financial arithmetic and UI state machines.
- Ensures all API and validation failures carry authoritative action tokens and surface directly in UI error boundaries.
- Prevents compilation errors and template crashes in Svelte 5 rune components.
- Enforces strict parity between Rust backend domain invariants and frontend presentation models.
