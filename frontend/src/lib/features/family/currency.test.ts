import { describe, it, expect } from "vitest";
import {
  isValidCurrencyCode,
  getCurrencyScale,
  getCurrencySymbol,
  formatMoney,
  formatCurrencyMajor,
  getBrowserRegion,
} from "./currency";
import { MOCK_SUPPORTED_CURRENCIES, resolveMockDefaultCurrency } from "./mock";

describe("currency helpers (Pure Domain & Formatting)", () => {
  it("includes major global currencies in mock data with zero ethnocentric bias", () => {
    const codes = MOCK_SUPPORTED_CURRENCIES.map((c) => c.code);
    expect(codes).toContain("INR");
    expect(codes).toContain("USD");
    expect(codes).toContain("EUR");
    expect(codes).toContain("GBP");
    expect(codes).toContain("JPY");
    expect(codes).toContain("CAD");
    expect(codes).toContain("AUD");
  });

  it("validates valid ISO-4217 currency codes", () => {
    expect(isValidCurrencyCode("USD")).toBe(true);
    expect(isValidCurrencyCode("inr")).toBe(true);
    expect(isValidCurrencyCode("EUR")).toBe(true);
    expect(isValidCurrencyCode("JPY")).toBe(true);
    expect(isValidCurrencyCode("INVALID")).toBe(false);
    expect(isValidCurrencyCode("")).toBe(false);
  });

  it("returns correct scale and symbol via native Intl", () => {
    expect(getCurrencyScale("USD")).toBe(2);
    expect(getCurrencyScale("INR")).toBe(2);
    expect(getCurrencyScale("JPY")).toBe(0);
    expect(getCurrencyScale("KRW")).toBe(0);

    expect(getCurrencySymbol("USD")).toBe("$");
    expect(getCurrencySymbol("INR")).toBe("₹");
    expect(getCurrencySymbol("EUR")).toBe("€");
    expect(getCurrencySymbol("GBP")).toBe("£");
    expect(getCurrencySymbol("JPY")).toBe("¥");
  });

  it("formats integer cents cleanly using native Intl formatting", () => {
    const usd = formatMoney(1999, "USD");
    expect(usd).toContain("19.99");

    const inr = formatMoney(1999, "INR");
    expect(inr).toContain("19.99");

    const jpy = formatMoney(500, "JPY");
    expect(jpy).toContain("500");

    const zero = formatMoney(0, "EUR");
    expect(zero).toContain("0.00");
  });

  it("formats major units without minor decimals", () => {
    const major = formatCurrencyMajor(1000, "USD");
    expect(major).toContain("1,000");
  });

  it("extracts browser region cleanly without currency bias", () => {
    const region = getBrowserRegion();
    if (region !== undefined) {
      expect(typeof region).toBe("string");
      expect(region.length).toBeGreaterThan(0);
    }
  });

  it("resolves mock default currency based on region without assuming USD", () => {
    expect(resolveMockDefaultCurrency("IN")).toBe("INR");
    expect(resolveMockDefaultCurrency("GB")).toBe("GBP");
    expect(resolveMockDefaultCurrency("DE")).toBe("EUR");
    expect(resolveMockDefaultCurrency("JP")).toBe("JPY");
    expect(resolveMockDefaultCurrency("US")).toBe("USD");
  });
});
