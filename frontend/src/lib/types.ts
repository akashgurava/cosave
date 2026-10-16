/**
 * Core domain primitives, nominal branding types, and invariant assertions.
 *
 * Provides compile-time branded types (UserId, MinorUnits, etc.) to prevent primitive
 * obsession, runtime smart constructors, discriminated AsyncState unions, and
 * expectPresent assertions with unique SCREAMING action tokens.
 */

import type { ErrorPayload } from "./api/contracts";
import { ParseEnumError } from "./errors";

export { ParseEnumError };

declare const __brand: unique symbol;

/**
 * Compile-time zero-cost nominal brand for creating distinct types from primitives.
 * Mirrors Rust's tuple struct newtypes (`struct UserId(String)`).
 */
export type Brand<T, B extends string> = T & { readonly [__brand]: B };

/**
 * Thrown when an unknown value fails conversion or parsing to a domain type or branded primitive.
 */
export class TypeConversionError extends Error {
  public override readonly name: string = "TypeConversionError";
  public readonly action: string = "UI.TYPE_CONVERSION.UNEXPECTED_FAILURE";
  public readonly targetType: string;
  public readonly raw: unknown;

  constructor(targetType: string, raw: unknown) {
    super(
      `Type: ${targetType}. Value: ${String(raw)}. Value cannot be converted to ${targetType}.`,
    );
    this.targetType = targetType;
    this.raw = raw;
  }
}

/**
 * Generic validator and converter for any branded positive integer ID.
 *
 * @throws {TypeConversionError} If raw is not a positive integer.
 */
export function toIntId<const B extends string>(brand: B, raw: unknown): Brand<number, B> {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new TypeConversionError(brand, raw);
  }
  return raw as Brand<number, B>;
}

/**
 * Generic validator and converter for any branded non-empty string ID.
 *
 * @throws {TypeConversionError} If raw is not a non-empty string.
 */
export function toStringId<const B extends string>(brand: B, raw: unknown): Brand<string, B> {
  if (typeof raw === "string" && raw.trim().length > 0) {
    return raw.trim() as Brand<string, B>;
  }
  throw new TypeConversionError(brand, raw);
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
    throw new TypeConversionError("MinorUnits", raw);
  }
  return raw as MinorUnits;
};

/** Sentinel value for filtering transactions without a subcategory */
export const UNASSIGNED_SUBCATEGORY_ID = 0 as SubcategoryId;

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
 * Thrown when an invariant expectation fails (e.g. missing relational entity).
 * Carries a unique SCREAMING action token pinpointing the exact failure site.
 */
export class InvariantViolationError extends Error {
  public readonly action: string;

  constructor(action: string, message: string) {
    super(`[${action}] InvariantViolation: ${message}`);
    this.name = "InvariantViolationError";
    this.action = action;
  }
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

/** Type helper to extract the literal values from an enum-like object */
export type EnumValues<T extends Record<string, string>> = T[keyof T];

/**
 * An immutable enum object with:
 * - Direct property access for each variant (e.g. `AccountType.BankAccount`)
 * - `$name`: Name of the enum for error messages
 * - `values`: Array of all allowed string values
 * - `parse(raw)`: Checks that an unknown value is a valid variant, or throws ParseEnumError
 */
export type EnumObject<T extends Record<string, string>> = Readonly<T> & {
  readonly $name: string;
  readonly values: readonly EnumValues<T>[];
  parse(raw: unknown): EnumValues<T>;
};

/**
 * Creates an enum object with variant properties, an allowed values list, and a `.parse(raw)` method.
 */
export function createEnum<const T extends Record<string, string>>(
  name: string,
  variants: T,
): EnumObject<T> {
  const allowed = Object.freeze(Object.values(variants)) as readonly EnumValues<T>[];

  const enumObj = {
    ...variants,
    $name: name,
    values: allowed,
    parse(raw: unknown): EnumValues<T> {
      if (typeof raw === "string" && (allowed as readonly string[]).includes(raw)) {
        return raw as EnumValues<T>;
      }
      throw new ParseEnumError(name, allowed, raw);
    },
  };

  return Object.freeze(enumObj) as EnumObject<T>;
}
