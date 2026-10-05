import { toMinorUnits, type MinorUnits } from "$lib/types/core";
import type { AmountCents, CurrencyCode, CurrencyOption } from "./types";

/**
 * Extracts the user's regional country/locale code from the browser without making currency assumptions.
 * Used when querying the backend for localized defaults (e.g. GET /api/v1/config/currency/default?region=IN).
 */
export function getBrowserRegion(): string | undefined {
  if (typeof window === "undefined" || typeof navigator === "undefined") {
    return undefined;
  }
  try {
    const lang = navigator.language ?? "";
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
  amountCents: AmountCents | number,
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

/**
 * Parses user input (string or number) into integer minor units according to currency scale.
 * Uses decimal string splitting to guarantee float-precision safety (e.g. "19.99" -> 1999).
 * Supports both dot '.' and comma ',' as decimal separators.
 */
export function parseMoneyInput(input: string | number, scale: number): MinorUnits {
  const str = String(input).trim().replace(",", ".");
  if (str === "" || str === "0") {
    return toMinorUnits(0);
  }

  const isNegative = str.startsWith("-");
  const cleaned = isNegative ? str.slice(1) : str;

  const parts = cleaned.split(".");
  const rawInteger = parts[0] ?? "";
  const integerPartDigits = rawInteger.replace(/\D/g, "");
  if (integerPartDigits === "" && parts.length === 1) {
    return toMinorUnits(0);
  }

  const integerVal = integerPartDigits === "" ? 0 : parseInt(integerPartDigits, 10);
  if (Number.isNaN(integerVal)) {
    return toMinorUnits(0);
  }

  if (scale <= 0) {
    return toMinorUnits(isNegative ? -integerVal : integerVal);
  }

  const fractionPartStr = (parts[1] ?? "").replace(/\D/g, "");
  const paddedFraction = (fractionPartStr + "0".repeat(scale)).slice(0, scale);
  const fractionVal = parseInt(paddedFraction, 10) || 0;

  const totalMinor = integerVal * 10 ** scale + fractionVal;
  return toMinorUnits(isNegative ? -totalMinor : totalMinor);
}

/**
 * Formats integer minor units into an editable decimal string suitable for form inputs.
 * Guarantees exact fractional digits matching the currency scale.
 */
export function formatMoneyInput(amount: MinorUnits | number, scale: number): string {
  if (scale <= 0) {
    return String(Math.trunc(amount));
  }
  const isNegative = amount < 0;
  const absAmount = Math.abs(Math.trunc(amount));
  const divisor = 10 ** scale;
  const integerPart = Math.floor(absAmount / divisor);
  const fractionPart = absAmount % divisor;
  const paddedFraction = String(fractionPart).padStart(scale, "0");
  const formatted = `${integerPart}.${paddedFraction}`;
  return isNegative ? `-${formatted}` : formatted;
}
