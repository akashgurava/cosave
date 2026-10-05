import { describe, it, expect } from "vitest";
import {
  getCurrencyScale,
  getCurrencySymbol,
  formatMoney,
  formatCurrencyMajor,
  getBrowserRegion,
  parseMoneyInput,
  formatMoneyInput,
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

  it("formats integer minor units cleanly using native Intl formatting", () => {
    const usd = formatMoney(1999, "USD");
    expect(usd).toContain("19.99");

    const inr = formatMoney(1999, "INR");
    expect(inr).toContain("19.99");

    const jpy = formatMoney(500, "JPY");
    expect(jpy).toContain("500");

    const zero = formatMoney(0, "EUR");
    expect(zero).toContain("0.00");
  });

  it("formats integer minor units with backend CurrencyOption", () => {
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

  describe("parseMoneyInput (Scale-Aware & Float-Safe Parsing)", () => {
    it("parses scale 2 (USD/EUR/INR) correctly with decimal and comma", () => {
      expect(parseMoneyInput("10.50", 2)).toBe(1050);
      expect(parseMoneyInput("10,50", 2)).toBe(1050);
      expect(parseMoneyInput("10.5", 2)).toBe(1050);
      expect(parseMoneyInput("10", 2)).toBe(1000);
      expect(parseMoneyInput("0", 2)).toBe(0);
      expect(parseMoneyInput("", 2)).toBe(0);
      expect(parseMoneyInput("  ", 2)).toBe(0);
    });

    it("prevents IEEE 754 floating point drift on values like 19.99", () => {
      expect(parseMoneyInput("19.99", 2)).toBe(1999);
      expect(parseMoneyInput("29.99", 2)).toBe(2999);
      expect(parseMoneyInput("0.07", 2)).toBe(7);
      expect(parseMoneyInput("0.1", 2)).toBe(10);
    });

    it("parses scale 0 (JPY/KRW) without decimal scaling", () => {
      expect(parseMoneyInput("500", 0)).toBe(500);
      expect(parseMoneyInput("500.99", 0)).toBe(500); // Truncates sub-unit input for scale 0
      expect(parseMoneyInput("0", 0)).toBe(0);
    });

    it("parses scale 3 (KWD/BHD) with 3 minor unit decimals", () => {
      expect(parseMoneyInput("1.250", 3)).toBe(1250);
      expect(parseMoneyInput("1.25", 3)).toBe(1250);
      expect(parseMoneyInput("1.2", 3)).toBe(1200);
      expect(parseMoneyInput("1", 3)).toBe(1000);
      expect(parseMoneyInput("0.005", 3)).toBe(5);
    });

    it("handles invalid or non-numeric strings safely by returning 0", () => {
      expect(parseMoneyInput("invalid", 2)).toBe(0);
      expect(parseMoneyInput("$$$", 2)).toBe(0);
    });
  });

  describe("formatMoneyInput (Scale-Aware Input String Formatting)", () => {
    it("formats scale 2 minor units into editable decimal string", () => {
      expect(formatMoneyInput(1050, 2)).toBe("10.50");
      expect(formatMoneyInput(1999, 2)).toBe("19.99");
      expect(formatMoneyInput(0, 2)).toBe("0.00");
    });

    it("formats scale 0 minor units without decimals", () => {
      expect(formatMoneyInput(500, 0)).toBe("500");
      expect(formatMoneyInput(0, 0)).toBe("0");
    });

    it("formats scale 3 minor units with 3 decimal places", () => {
      expect(formatMoneyInput(1250, 3)).toBe("1.250");
      expect(formatMoneyInput(5, 3)).toBe("0.005");
      expect(formatMoneyInput(0, 3)).toBe("0.000");
    });
  });
});
