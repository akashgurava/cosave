import { ContractViolationError, type ErrorPayload } from "$lib/api";

declare const __brand: unique symbol;

/**
 * Compile-time zero-cost nominal brand for creating distinct types from primitives.
 * Mirrors Rust's tuple struct newtypes (`struct UserId(String)`).
 */
export type Brand<T, B extends string> = T & { readonly [__brand]: B };

export type UserId = Brand<string, "UserId">;
export type FamilyId = Brand<string, "FamilyId">;
export type CategoryId = Brand<string, "CategoryId">;
export type SubcategoryId = Brand<string, "SubcategoryId">;
export type AmountCents = Brand<number, "AmountCents">;

/**
 * Validates and converts an unknown value to a branded AmountCents integer.
 * Monetary values must strictly be integer cents, never floating-point.
 */
export function toAmountCents(raw: unknown): AmountCents {
  if (typeof raw !== "number" || !Number.isInteger(raw)) {
    throw new ContractViolationError("AmountCents must be an integer", raw);
  }
  return raw as AmountCents;
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
