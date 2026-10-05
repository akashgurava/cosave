import { describe, it, expect } from "vitest";
import {
  getCurrencyScale,
  getCurrencySymbol,
  formatMoney,
  formatCurrencyMajor,
  getBrowserRegion,
} from "./currency";
import type { CurrencyId, CurrencyOption } from "./types";

describe("currency helpers (Pure Presentation & Formatting)", () => {
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

  it("respects backend CurrencyOption overrides for scale and symbol", () => {
    const customInr: CurrencyOption = {
      id: 3 as CurrencyId,
      code: "INR",
      name: "Indian Rupee",
      symbol: "₹",
      scale: 2,
    };
    expect(getCurrencyScale("INR", customInr)).toBe(2);
    expect(getCurrencySymbol("INR", customInr)).toBe("₹");

    const customJpy: CurrencyOption = {
      id: 4 as CurrencyId,
      code: "JPY",
      name: "Japanese Yen",
      symbol: "¥",
      scale: 0,
    };
    expect(getCurrencyScale("JPY", customJpy)).toBe(0);
    expect(getCurrencySymbol("JPY", customJpy)).toBe("¥");
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

  it("formats integer cents with backend CurrencyOption", () => {
    const inrOption: CurrencyOption = {
      id: 3 as CurrencyId,
      code: "INR",
      name: "Indian Rupee",
      symbol: "₹",
      scale: 2,
    };
    const formatted = formatMoney(500000, "INR", inrOption);
    expect(formatted).toContain("5,000.00");
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
});
