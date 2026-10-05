# TypeScript Compiler Strictness and Anti-Looseness Invariants

The frontend enforces maximum TypeScript compiler strictness, bans runtime-unsafe language quirks (`any`, blind `as T` casting, implicit truthy/falsy coercion), and leverages compile-time nominal branding to mirror Rust's type rigidity.

## Context & Decision

JavaScript and standard TypeScript permit loose duck typing, structural type equivalence, unchecked array indexing, and implicit type coercion. In a financial and collaborative platform, these loose defaults cause silent bugs (e.g. `0` treated as falsy, `arr[i]` returning `undefined` when typed as `T`, or a `UserId` mistakenly passed where a `CategoryId` is expected).

To establish Rust-grade guarantees, we enforce:

1. **Maximum Compiler Invariants (`tsconfig.json`)**:
   - `noUncheckedIndexedAccess: true`: Indexing arrays (`arr[i]`) or dictionary records (`dict[key]`) returns `T | undefined`, forcing callers to explicitly check existence before use (mirroring Rust's `Option<&T>`).
   - `noImplicitReturns: true`: Every function code branch must explicitly return a typed value or throw.
   - `noFallthroughCasesInSwitch: true`: Every non-empty `switch` case must terminate with `break`, `return`, or `throw`.
   - `noImplicitOverride: true`: Subclasses overriding base class methods must use the explicit `override` keyword.
   - *Ecosystem Trade-off on `exactOptionalPropertyTypes`*: Left disabled in `tsconfig.json` because upstream UI libraries (`bits-ui` / shadcn-svelte) and standard DOM types (`RequestInit`) produce union-overflow errors. In internal domain interfaces, developers explicitly annotate `prop?: T | undefined` when `undefined` is permitted.

2. **Zero `any` Policy**:
   - The `any` type is strictly forbidden across all production code and test files. Untyped external data must be received as `unknown` and narrowed using runtime type predicates or schema decoders.

3. **Zero Blind `as T` Type Assertions**:
   - Blind type casting (`const user = res as User`) is prohibited. Casting bypasses the compiler and lies to the runtime.
   - Data crossing any boundary must be validated via runtime decoders (`parseX(raw: unknown)`).
   - The only allowed uses of `as` are `as const` (for defining literal values and immutable arrays) and narrowing inside test assertions.

4. **Ban Implicit Truthy/Falsy Coercion**:
   - JavaScript's implicit truthiness checks (`if (count)`, `if (name)`) introduce severe bugs when values are `0`, `""`, or `false`.
   - Conditionals must be explicit:
     - Numeric checks: `count > 0`, `count !== 0`, or `Number.isInteger(count)`
     - String checks: `name.length > 0` or `name.trim() !== ""`
     - Nullability checks: `val !== null && val !== undefined` or nullish coalescing `??`.

5. **Nominal Branding (Rust "Newtypes" in TypeScript)**:
   - TypeScript's structural typing allows swapping string IDs or integer quantities indiscriminately.
   - We enforce zero-cost compile-time branding:
     ```ts
     declare const __brand: unique symbol;
     export type Brand<T, B extends string> = T & { readonly [__brand]: B };

     export type UserId = Brand<string, "UserId">;
     export type CategoryId = Brand<number, "CategoryId">;
     export type AmountCents = Brand<number, "AmountCents">;
     ```
   - Branded types are instantiated through smart constructor functions that validate inputs at runtime and return branded types.

## Consequences

- Prevents runtime crashes from unexpected `undefined` index access and implicit type coercion.
- Structural swapping of IDs (e.g. passing `CategoryId` to `UserId`) is caught at compile-time.
- Eliminates the false sense of security caused by blind `as T` casting.
