import type { CurrencyCode, CurrencyOption } from "./types";

/**
 * Extracts the user's regional country/locale code from the browser without making currency assumptions.
 * Used when querying the backend for localized defaults (e.g. GET /api/v1/config/currency/default?region=IN).
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
 * Resolves minor unit decimal scale for a currency via backend metadata or native Intl.
 */
export function getCurrencyScale(currency: CurrencyCode, currencyOption?: CurrencyOption): number {
  if (currencyOption !== undefined) {
    return currencyOption.scale;
  }
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
 * Resolves native currency symbol via backend metadata or native Intl formatToParts.
 */
export function getCurrencySymbol(currency: CurrencyCode, currencyOption?: CurrencyOption): string {
  if (currencyOption !== undefined) {
    return currencyOption.symbol;
  }
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
 * Formats an integer amount (in minor units / cents) using native Intl engine and backend metadata.
 * Requires an explicit CurrencyCode — zero hardcoded defaults.
 */
export function formatMoney(
  amountCents: number,
  currency: CurrencyCode,
  currencyOption?: CurrencyOption,
): string {
  const scale = getCurrencyScale(currency, currencyOption);
  const major = scale === 0 ? amountCents : amountCents / 10 ** scale;
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      minimumFractionDigits: scale,
      maximumFractionDigits: scale,
    }).format(major);
  } catch {
    const sym = getCurrencySymbol(currency, currencyOption);
    return `${sym}${major.toFixed(scale)}`;
  }
}

/**
 * Formats a major units value (e.g. for rounded badge/summary display) with the currency symbol.
 * Requires an explicit CurrencyCode — zero hardcoded defaults.
 */
export function formatCurrencyMajor(
  majorValue: number,
  currency: CurrencyCode,
  currencyOption?: CurrencyOption,
): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(majorValue);
  } catch {
    const sym = getCurrencySymbol(currency, currencyOption);
    return `${sym}${majorValue.toLocaleString()}`;
  }
}
