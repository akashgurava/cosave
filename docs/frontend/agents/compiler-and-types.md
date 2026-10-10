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
  if (isObject(data) === false) throw new Error("Expected object");
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

### Explicit Truthy/Falsy Checks & Strict Equality
Never rely on implicit truthiness. JavaScript coercion introduces subtle bugs around `0`, empty strings, and booleans:
```ts
// FORBIDDEN
if (count) { ... }
if (name) { ... }
if (isOpen) { ... }
if (!isOpen) { ... }

// REQUIRED
if (count > 0) { ... }
if (name.length > 0) { ... }
if (isOpen === true) { ... }
if (isOpen === false) { ... }
if (value !== null && value !== undefined) { ... }
```

### Zero Loose Operators (`??`, `?.`, `!`)
Loose operators mask uninitialized states, bypass compiler safety, and degrade domain contracts:
- **Zero Loose `??`**: Do not use `??` for default liquidations or synthetic domain fallbacks (`?? ""`, `?? {}`). Use explicit branching or invariant getters (`expectPresent`).
- **Zero Unchecked `?.`**: Do not chain optional access across domain models. If a schema property is required, access it directly. If optional, branch explicitly.
- **Zero Non-Null Assertions (`!`)**: The `!` assertion operator is strictly banned across product and test code. Narrow types using runtime guards.
Follow [`docs/frontend/adr/0007-zero-tolerance-anti-looseness-and-explicit-branching.md`](../adr/0007-zero-tolerance-anti-looseness-and-explicit-branching.md).

### Zero Defensive Type Liquidation & Bogus Fallbacks
Never mask invariant foreign key relationships with defensive optional chaining (`?.`), nullish coalescing to arbitrary defaults (`?? "Custom"`, `?? ""`), or synthesized fake objects:
```ts
// FORBIDDEN: Synthesizing fake domain objects to silence compiler nullability
const currency = currencies.find(c => c.id === account?.currencyId) ?? {
  id: 1 as CurrencyId,
  code: "USD",
  name: "US Dollar",
  symbol: "$",
  scale: 2,
};

// REQUIRED: Invariant assertion via store getter (fails fast on data defects)
const currency = familyStore.getCurrency(account.currencyId);
```
If a relationship is genuinely optional in the domain, model it explicitly as `T | null` at the schema and decoder level. If an association is required by database foreign keys, missing data is an invariant violation, not an optional value.

### Invariant Assertions (`expectPresent`)
`expectPresent(val, action, message)` is not a fallback mechanism or default liquidator. It asserts an authoritative **domain invariant**: **we expect a value to be present at this exact point in execution.** If the value is `null` or `undefined`, the application is in an illegal state, and `expectPresent` immediately raises an `InvariantViolationError` with a unique compile-time action token.

```ts
// REQUIRED: Assert invariant prerequisites directly at the boundary
const family = expectPresent(
  this.family,
  "CONFIG.FAMILY.ADD_MEMBER",
  "Cannot add member without an initialized family",
);
const familyId = family.id;

// FORBIDDEN: Wrapping expectPresent inside fallback ternaries
const name = input.name !== undefined ? input.name : expectPresent(this.family, ...).name;

// FORBIDDEN: Defensive fallback ternaries for inputs that are already required
const familyName = input.familyName !== undefined ? input.familyName : family.familyName;
```
For genuinely optional parameters, branch explicitly (`if (input.name !== undefined)`). When an input property is required by its TypeScript contract, pass it directly—TypeScript enforces presence at compile time and the backend validates at runtime. Reserve `expectPresent` strictly for invariant prerequisites whose absence represents a broken contract or invalid state. Never bury `expectPresent` inside ternary operators as a liquidator.


## 3. Nominal Branding (Rust Newtypes)

Prevent structural equivalence bugs between domain IDs and financial units using `Brand<T, Tag>`:

```ts
import type { Brand } from "$lib/types/core";

export type UserId = Brand<string, "UserId">;
export type CategoryId = Brand<number, "CategoryId">;
export type MinorUnits = Brand<number, "MinorUnits">;

// Constructor / Parse helper
export function toMinorUnits(raw: unknown): MinorUnits {
  if (typeof raw !== "number" || Number.isInteger(raw) === false) {
    throw new ContractViolationError("MinorUnits must be an integer", raw);
  }
  return raw as MinorUnits;
}
```

Never treat raw numbers as money; always wrap in scale-aware integer `MinorUnits`.

### Nominal Branding Integrity (Zero Blind `as Brand`)
Do not bypass nominal branding with blind assertions (`id as CategoryId`, `raw as MinorUnits`) in view templates, event handlers, or modals. Branded values must originate exclusively from:
1. Runtime schema decoders (`parseX(raw)`)
2. Smart constructors with validation (`toMinorUnits(raw)`)
3. Strongly-typed store properties and models (`item.id`)
If an event handler receives a selection from an HTML `<select>` or radio group, narrow or parse it via the store's typed entity list rather than asserting `val as Brand`.

