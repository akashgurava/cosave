/**
 * Core domain primitives, nominal branding types, and invariant assertions.
 *
 * Provides compile-time branded types (UserId, MinorUnits, etc.) to prevent primitive
 * obsession, runtime smart constructors, discriminated AsyncState unions, and
 * expectPresent assertions with unique SCREAMING action tokens.
 */

import { ContractViolationError, type ErrorPayload } from "./api/contracts";

declare const __brand: unique symbol;

/**
 * Compile-time zero-cost nominal brand for creating distinct types from primitives.
 * Mirrors Rust's tuple struct newtypes (`struct UserId(String)`).
 */
export type Brand<T, B extends string> = T & { readonly [__brand]: B };

export type UserId = Brand<string, "UserId">;
export type FamilyId = Brand<number, "FamilyId">;
export type MemberId = Brand<number, "MemberId">;
export type AccountId = Brand<number, "AccountId">;
export type TypeId = Brand<number, "TypeId">;
export type CategoryId = Brand<number, "CategoryId">;
export type SubcategoryId = Brand<number, "SubcategoryId">;
export type TransactionId = Brand<string, "TransactionId">;
export type CurrencyId = Brand<number, "CurrencyId">;
export type MinorUnits = Brand<number, "MinorUnits">;

export interface Currency {
  readonly code: string;
  readonly scale: number;
  readonly symbol?: string;
}

/**
 * Validates and converts an unknown value to a branded UserId string.
 */
export function toUserId(raw: unknown): UserId {
  if (typeof raw !== "string" || raw.trim().length === 0) {
    throw new ContractViolationError("UserId must be a non-empty string", raw);
  }
  return raw as UserId;
}

/**
 * Validates and converts an unknown value to a branded TransactionId string.
 */
export function toTransactionId(raw: unknown): TransactionId {
  if (typeof raw === "string" && raw.trim().length > 0) {
    return raw.trim() as TransactionId;
  }
  if (typeof raw === "number" && Number.isInteger(raw) && raw > 0) {
    return String(raw) as TransactionId;
  }
  throw new ContractViolationError("TransactionId must be a non-empty string", raw);
}

/**
 * Validates and converts an unknown value to a branded FamilyId integer.
 */
export function toFamilyId(raw: unknown): FamilyId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("FamilyId must be a positive integer", raw);
  }
  return raw as FamilyId;
}

/**
 * Validates and converts an unknown value to a branded MemberId integer.
 */
export function toMemberId(raw: unknown): MemberId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("MemberId must be a positive integer", raw);
  }
  return raw as MemberId;
}

/**
 * Validates and converts an unknown value to a branded AccountId integer.
 */
export function toAccountId(raw: unknown): AccountId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("AccountId must be a positive integer", raw);
  }
  return raw as AccountId;
}

/**
 * Validates and converts an unknown value to a branded CurrencyId integer.
 */
export function toCurrencyId(raw: unknown): CurrencyId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("CurrencyId must be a positive integer", raw);
  }
  return raw as CurrencyId;
}

/**
 * Validates and converts an unknown value to a branded TypeId integer.
 */
export function toTypeId(raw: unknown): TypeId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("TypeId must be a positive integer", raw);
  }
  return raw as TypeId;
}

/**
 * Validates and converts an unknown value to a branded CategoryId integer.
 */
export function toCategoryId(raw: unknown): CategoryId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("CategoryId must be a positive integer", raw);
  }
  return raw as CategoryId;
}

/**
 * Validates and converts an unknown value to a branded SubcategoryId integer.
 */
export function toSubcategoryId(raw: unknown): SubcategoryId {
  if (typeof raw !== "number" || Number.isInteger(raw) === false || raw <= 0) {
    throw new ContractViolationError("SubcategoryId must be a positive integer", raw);
  }
  return raw as SubcategoryId;
}

/**
 * Validates and converts an unknown value to a branded MinorUnits integer.
 * Monetary values must strictly be integer minor units, never floating-point.
 */
export function toMinorUnits(raw: unknown): MinorUnits {
  if (typeof raw !== "number" || Number.isInteger(raw) === false) {
    throw new ContractViolationError("MinorUnits must be an integer", raw);
  }
  return raw as MinorUnits;
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
