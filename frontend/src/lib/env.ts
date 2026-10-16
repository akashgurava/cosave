/**
 * Browser runtime and DOM environment capability detection helpers.
 *
 * Provides safe guards for window, document, localStorage, and mediaQuery availability
 * across client, server (SSR), and test environments.
 */

import { expectPresent } from "./types";

/**
 * Returns true strictly when running in a runtime where window is defined.
 */
export function hasWindow(): boolean {
  return typeof window !== "undefined";
}

/**
 * Asserts and returns window in a browser environment.
 *
 * @param action - Dedicated compile-time action token pinpointing the caller site.
 * @throws {InvariantViolationError} If window is undefined.
 */
export function expectWindow(action: string): Window & typeof globalThis {
  return expectPresent(
    typeof window !== "undefined" ? window : null,
    action,
    "window must be present in a browser environment",
  );
}

/**
 * Returns true strictly when localStorage is accessible in the runtime environment.
 */
export function hasLocalStorage(): boolean {
  return typeof localStorage !== "undefined";
}

/**
 * Returns true strictly when window and window.matchMedia function are available.
 */
export function hasMatchMedia(): boolean {
  return typeof window !== "undefined" && typeof window.matchMedia === "function";
}

/**
 * Returns true strictly when document is defined.
 */
export function hasDocument(): boolean {
  return typeof document !== "undefined";
}

/**
 * Returns true strictly when document.documentElement is defined and non-null.
 */
export function hasDocumentElement(): boolean {
  return (
    typeof document !== "undefined" &&
    document.documentElement !== null &&
    document.documentElement !== undefined
  );
}

/**
 * Asserts and returns document.documentElement in a DOM environment.
 *
 * @param action - Dedicated compile-time action token pinpointing the caller site.
 * @throws {InvariantViolationError} If document or document.documentElement is missing.
 */
export function expectDocumentElement(action: string): HTMLElement {
  const root =
    typeof document !== "undefined" &&
    document.documentElement !== null &&
    document.documentElement !== undefined
      ? document.documentElement
      : null;

  return expectPresent(
    root,
    action,
    "document.documentElement must be present in a DOM environment",
  );
}
