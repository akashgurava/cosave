/**
 * Core domain primitives, nominal branding types, and invariant assertions.
 *
 * Provides compile-time branded types (UserId, MinorUnits, etc.) to prevent primitive
 * obsession, runtime smart constructors, discriminated AsyncState unions, and
 * expectPresent assertions with unique SCREAMING action tokens.
 */

import { z } from "zod";
import type { ErrorPayload } from "./api/contracts";
import {
  ParseModelError,
  ParseEnumError,
  normalizeZodIssue,
  IdCreationError,
  InvariantViolationError,
} from "./errors";

declare const __brand: unique symbol;

/**
 * Nominal brand tag creating distinct compile-time types from primitives.
 */
export type Brand<T, B extends string> = T & { readonly [__brand]: B };

/**
 * Generic validator and converter for any branded positive integer ID.
 *
 * @throws {IdCreationError} If raw is not a positive integer.
 */
export function toIntId<const B extends string>(brand: B, raw: unknown): Brand<number, B> {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new IdCreationError("number", brand, raw);
  }
  return raw as Brand<number, B>;
}

/**
 * Generic validator and converter for any branded non-empty string ID.
 *
 * @throws {IdCreationError} If raw is not a non-empty string.
 */
export function toStringId<const B extends string>(brand: B, raw: unknown): Brand<string, B> {
  if (typeof raw === "string" && raw.trim().length > 0) {
    return raw.trim() as Brand<string, B>;
  }
  throw new IdCreationError("string", brand, raw);
}

/**
 * Extracts and trims a text value from FormData or FormDataEntryValue.
 * Returns an empty string if missing or non-string.
 */
export function getTextFormValue(formData: FormData, fieldName: string): string {
  const val = formData.get(fieldName);
  return typeof val === "string" ? val.trim() : "";
}

// =============================================================================
// Domain Branded Types & Companion Smart Constructors
// =============================================================================

export type UserId = Brand<string, "UserId">;
export const UserId = (raw: unknown): UserId => toStringId("UserId", raw);

export type FamilyId = Brand<number, "FamilyId">;
export const FamilyId = (raw: unknown): FamilyId => toIntId("FamilyId", raw);

export type MemberId = Brand<number, "MemberId">;
export const MemberId = (raw: unknown): MemberId => toIntId("MemberId", raw);

export type AccountId = Brand<number, "AccountId">;
export const AccountId = (raw: unknown): AccountId => toIntId("AccountId", raw);

export type TypeId = Brand<number, "TypeId">;
export const TypeId = (raw: unknown): TypeId => toIntId("TypeId", raw);

export type CategoryId = Brand<number, "CategoryId">;
export const CategoryId = (raw: unknown): CategoryId => toIntId("CategoryId", raw);

export type SubcategoryId = Brand<number, "SubcategoryId">;
export const SubcategoryId = (raw: unknown): SubcategoryId => toIntId("SubcategoryId", raw);

export type ColorId = Brand<number, "ColorId">;
export const ColorId = (raw: unknown): ColorId => toIntId("ColorId", raw);

export type TransactionId = Brand<string, "TransactionId">;
export const TransactionId = (raw: unknown): TransactionId => toStringId("TransactionId", raw);

export type CurrencyId = Brand<number, "CurrencyId">;
export const CurrencyId = (raw: unknown): CurrencyId => toIntId("CurrencyId", raw);

export type MinorUnits = Brand<number, "MinorUnits">;
export const MinorUnits = (raw: unknown): MinorUnits => {
  if (typeof raw !== "number" || Number.isInteger(raw) === false) {
    throw new IdCreationError("number", "MinorUnits", raw);
  }
  return raw as MinorUnits;
};

/** Sentinel value for filtering transactions without a subcategory */
export const UNASSIGNED_SUBCATEGORY_ID = 0 as SubcategoryId;

/** Supported currency display metadata */
export interface Currency {
  readonly code: string;
  readonly scale: number;
  readonly symbol: string | null;
}

/**
 * Discriminated union for asynchronous query/fetch operations.
 * Makes impossible visual states unrepresentable (e.g. loading while showing error).
 */
export type AsyncState<T, E = ErrorPayload> =
  | { readonly status: "idle" }
  | { readonly status: "loading" }
  | { readonly status: "success"; readonly data: T }
  | { readonly status: "error"; readonly error: E };

/**
 * Tagged union representing either success with a value or failure with an error.
 * Mirrors Rust's `Result<T, E>`.
 */
export type Result<T, E> =
  { readonly ok: true; readonly value: T } | { readonly ok: false; readonly error: E };

export function ok<T>(value: T): Result<T, never> {
  return { ok: true, value };
}

export function err<E>(error: E): Result<never, E> {
  return { ok: false, error };
}

/**
 * Asserts that a value is present (neither null nor undefined), returning guaranteed non-nullable T.
 *
 * ### Philosophy & Domain Invariants
 * `expectPresent` is NOT a fallback utility or error-suppression mechanism. It expresses
 * an authoritative domain expectation: **by system design, business rules, or database contract,
 * this value MUST be present at this execution point.**
 *
 * If the value is `null` or `undefined`, the application has entered an illegal state or violated
 * a foundational invariant. `expectPresent` fails fast by raising an `InvariantViolationError`
 * tagged with a unique compile-time SCREAMING action token pinpointing the exact failure site.
 *
 * ### When to use:
 * - Accessing relational entities guaranteed by foreign keys (e.g. member from ID, currency from ID).
 * - Accessing initialized store state that must exist prior to executing a command (e.g. `this.family`).
 * - Unwrapping an invariant value where absence indicates broken data integrity or programming error.
 *
 * ### When NOT to use:
 * - Do NOT use `expectPresent` to handle genuinely optional parameters or optional user inputs.
 *   Use explicit branching (`if (param !== undefined)`) for optional arguments.
 * - Do NOT bury `expectPresent` inside ternary fallback chains to paper over uninitialized state.
 *
 * Mirrors Rust's `Option::expect("...")` paired with strict compile-time action tracing.
 */
export function expectPresent<T>(val: T | null | undefined, action: string, message: string): T {
  if (val === null || val === undefined) {
    throw new InvariantViolationError(action, message);
  }
  return val;
}

/**
 * Recursively freezes an object or array to enforce in-memory immutability.
 */
export function deepFreeze<T>(val: T): Readonly<T> {
  if (val === null || typeof val !== "object" || Object.isFrozen(val)) {
    return val;
  }
  Object.freeze(val);
  for (const key of Object.keys(val)) {
    const prop = (val as Record<string, unknown>)[key];
    if (prop !== null && typeof prop === "object") {
      deepFreeze(prop);
    }
  }
  return val;
}

export type EnumValues<T extends Record<string, string>> = T[keyof T];

/**
 * An immutable enum object with:
 * - Direct property access for each variant (e.g. `AccountType.BankAccount`)
 * - `$name`: Name of the enum for error messages
 * - `schema`: Zod enum schema for composition inside model schemas
 * - `values`: Array of all allowed string values
 * - `parse(raw)`: Standalone parser throwing ParseEnumError
 */
export type EnumObject<T extends Record<string, string>> = Readonly<T> & {
  readonly $name: string;
  readonly schema: z.ZodType<EnumValues<T>>;
  readonly values: readonly EnumValues<T>[];
  parse(raw: unknown): EnumValues<T>;
};

/**
 * Creates an immutable enum object with variant properties, a Zod schema,
 * an allowed values list, and a standalone `.parse(raw)` method.
 */
export function createEnum<const T extends Record<string, string>>(
  name: string,
  variants: T,
): EnumObject<T> {
  const values = Object.values(variants) as [T[keyof T], ...T[keyof T][]];
  const schema = z.enum(values);

  const enumObj = {
    ...variants,
    $name: name,
    schema,
    values: Object.freeze(values) as readonly EnumValues<T>[],
    parse(raw: unknown): EnumValues<T> {
      if (typeof raw === "string" && (values as readonly string[]).includes(raw)) {
        return raw as EnumValues<T>;
      }
      throw new ParseEnumError(name, values as readonly string[], raw);
    },
  };

  return Object.freeze(enumObj) as EnumObject<T>;
}

/**
 * Authoritative centralized model parser.
 *
 * Validates unknown raw data against a Zod schema, returning deep-frozen data.
 * Collects all Zod validation issues and throws ParseModelError on failure.
 */
export function parseModel<T extends z.ZodTypeAny>(
  schema: T,
  raw: unknown,
  modelName: string,
): Readonly<z.infer<T>> {
  const result = schema.safeParse(raw);
  if (result.success) {
    return deepFreeze(result.data);
  }

  const issues = result.error.issues.map((i) => normalizeZodIssue(i, raw, modelName));
  throw new ParseModelError(modelName, issues, raw);
}

export function defineModel<T extends z.ZodTypeAny>(name: string, schema: T) {
  return Object.assign(schema, {
    modelName: name,
    parse(raw: unknown): Readonly<z.infer<T>> {
      return parseModel(schema, raw, name);
    },
  });
}
