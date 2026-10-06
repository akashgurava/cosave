import { ContractViolationError, type ErrorPayload } from "$lib/api";

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
export type CategoryId = Brand<number, "CategoryId">;
export type SubcategoryId = Brand<number, "SubcategoryId">;
export type TransactionId = Brand<number, "TransactionId">;
export type CurrencyId = Brand<number, "CurrencyId">;
export type MinorUnits = Brand<number, "MinorUnits">;

export interface Currency {
  readonly code: string;
  readonly scale: number;
  readonly symbol?: string;
}

/**
 * Validates and converts an unknown value to a branded TransactionId integer.
 */
export function toTransactionId(raw: unknown): TransactionId {
  if (typeof raw !== "number" || !Number.isInteger(raw) || raw <= 0) {
    throw new ContractViolationError("TransactionId must be a positive integer", raw);
  }
  return raw as TransactionId;
}

/**
 * Validates and converts an unknown value to a branded MinorUnits integer.
 * Monetary values must strictly be integer minor units, never floating-point.
 */
export function toMinorUnits(raw: unknown): MinorUnits {
  if (typeof raw !== "number" || !Number.isInteger(raw)) {
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
