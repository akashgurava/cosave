import type { CurrencyCode } from "./types";

/**
 * Validates whether a string is a standard 3-character ISO-4217 currency code.
 */
export function isValidCurrencyCode(code: string): code is CurrencyCode {
  return typeof code === "string" && /^[A-Za-z]{3}$/.test(code.trim());
}

/**
 * Extracts the user's regional country/locale code from the browser without making currency assumptions.
 * Used when querying the backend for localized defaults (e.g. GET /api/v1/currencies/default?region=IN).
 */
export function getBrowserRegion(): string | undefined {
  if (typeof window === "undefined" || typeof navigator === "undefined") {
    return undefined;
  }
  try {
    const lang = navigator.language || "";
    const parts = lang.split("-");
    return parts.length > 1 ? parts[parts.length - 1]?.toUpperCase() : undefined;
  } catch {
    return undefined;
  }
}

/**
 * Dynamically resolves minor unit decimal scale for any currency via native Intl.
 */
export function getCurrencyScale(currency: CurrencyCode): number {
  try {
    return (
      new Intl.NumberFormat(undefined, {
        style: "currency",
        currency,
      }).resolvedOptions().maximumFractionDigits ?? 2
    );
  } catch {
    return 2;
  }
}

/**
 * Dynamically extracts native currency symbol via native Intl formatToParts.
 */
export function getCurrencySymbol(currency: CurrencyCode): string {
  try {
    const parts = new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
    }).formatToParts(0);
    const symbolPart = parts.find((p) => p.type === "currency");
    return symbolPart?.value ?? currency;
  } catch {
    return currency;
  }
}

/**
 * Formats an integer amount (in minor units / cents) using native Intl engine.
 * Requires an explicit CurrencyCode — zero hardcoded defaults.
 */
export function formatMoney(amountCents: number, currency: CurrencyCode): string {
  const scale = getCurrencyScale(currency);
  const major = scale === 0 ? amountCents : amountCents / 10 ** scale;
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      minimumFractionDigits: scale,
      maximumFractionDigits: scale,
    }).format(major);
  } catch {
    return `${getCurrencySymbol(currency)}${major.toFixed(scale)}`;
  }
}

/**
 * Formats a major units value (e.g. for rounded badge/summary display) with the currency symbol.
 * Requires an explicit CurrencyCode — zero hardcoded defaults.
 */
export function formatCurrencyMajor(majorValue: number, currency: CurrencyCode): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(majorValue);
  } catch {
    return `${getCurrencySymbol(currency)}${majorValue.toLocaleString()}`;
  }
}
