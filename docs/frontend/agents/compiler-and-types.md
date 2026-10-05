# Frontend Compiler Rigidity & Type Standards

Guidelines for authoring TypeScript in the CoSave frontend with Rust-grade strictness.

## Trigger & Scope

Consult this guide whenever writing TypeScript code, defining data models, adding functions, or configuring compiler and linter rules in `frontend/`.

## 1. Compiler Invariants (`tsconfig.json`)

All frontend code must compile cleanly under strict compiler flags:

- **`noUncheckedIndexedAccess: true`**: Reading from arrays (`items[0]`) or indexed maps (`dict[key]`) returns `T | undefined`. You must explicitly check for `undefined` before accessing properties:
  ```ts
  // FORBIDDEN: Blind access
  const first = items[0];
  const name = first.name; // Error: Object is possibly 'undefined'

  // REQUIRED: Guarded access (Option<&T> mental model)
  const first = items[0];
  if (first === undefined) return null;
  const name = first.name;
  ```
- **Optional Properties Precision**: While `exactOptionalPropertyTypes` is disabled in `tsconfig.json` due to upstream UI library incompatibilities, domain code must explicitly annotate `prop?: T | undefined` when `undefined` can be passed.
- **`noImplicitReturns: true`**: Every control path must return a value or throw.
- **`noFallthroughCasesInSwitch: true`**: Switch cases must break or return.
- **`noImplicitOverride: true`**: Subclass method overrides must declare `override`.

## 2. Banned TypeScript Looseness

### Zero `any`
The `any` type is strictly forbidden. If external data type is unknown, use `unknown` and narrow it with runtime type guards or schema decoders:
```ts
// FORBIDDEN
function handle(data: any) { ... }

// REQUIRED
function handle(data: unknown) {
  if (!isObject(data)) throw new Error("Expected object");
  ...
}
```

### Zero Blind `as T` Assertions
Never bypass type checking by casting network responses, local storage, or DOM attributes as a type:
```ts
// FORBIDDEN
const user = (await res.json()) as User;

// REQUIRED: Validate at runtime
const raw: unknown = await res.json();
const user = parseUser(raw);
```
*Note*: `as const` is permitted for literal tuples/enums (e.g. `['a', 'b'] as const`). In unit tests, avoid casting partial mocks with `as User`; use complete test fixtures or `@total-typescript/shoehorn` (`import { partial } from "@total-typescript/shoehorn"`).

### Explicit Truthy/Falsy Checks
Never rely on implicit truthiness. JavaScript coercion introduces subtle bugs around `0` and empty strings:
```ts
// FORBIDDEN
if (count) { ... }
if (name) { ... }

// REQUIRED
if (count > 0) { ... }
if (name.length > 0) { ... }
if (value !== null && value !== undefined) { ... }
```

## 3. Nominal Branding (Rust Newtypes)

Prevent structural equivalence bugs between domain IDs and financial units using `Brand<T, Tag>`:

```ts
import type { Brand } from "$lib/types/core";

export type UserId = Brand<string, "UserId">;
export type CategoryId = Brand<number, "CategoryId">;
export type MinorUnits = Brand<number, "MinorUnits">;

// Constructor / Parse helper
export function toMinorUnits(raw: unknown): MinorUnits {
  if (typeof raw !== "number" || !Number.isInteger(raw)) {
    throw new ContractViolationError("MinorUnits must be an integer", raw);
  }
  return raw as MinorUnits;
}
```

Never treat raw numbers as money; always wrap in scale-aware integer `MinorUnits`.
