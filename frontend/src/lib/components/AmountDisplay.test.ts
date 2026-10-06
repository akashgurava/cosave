import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import AmountDisplay from "./AmountDisplay.svelte";
import type { Currency, MinorUnits } from "$lib/types";

describe("AmountDisplay Component (Tier 1 / Unit)", () => {
  const usdCurrency: Currency = Object.freeze({
    code: "USD",
    scale: 2,
    symbol: "$",
  });

  const jpyCurrency: Currency = Object.freeze({
    code: "JPY",
    scale: 0,
    symbol: "¥",
  });

  const kwdCurrency: Currency = Object.freeze({
    code: "KWD",
    scale: 3,
    symbol: "KD",
  });

  describe("Scale & Amount Formatting", () => {
    it("formats 2-decimal scale correctly (USD)", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 12550 as MinorUnits,
          currency: usdCurrency,
        },
      });
      // 12550 minor units with scale 2 = 125.50
      expect(body).toContain("125.50");
      expect(body).toContain("$");
    });

    it("formats 0-decimal scale correctly (JPY)", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 5000 as MinorUnits,
          currency: jpyCurrency,
        },
      });
      // 5000 minor units with scale 0 = 5000
      expect(body).toContain("5,000");
      expect(body).toContain("¥");
    });

    it("formats 3-decimal scale correctly (KWD)", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 15750 as MinorUnits,
          currency: kwdCurrency,
        },
      });
      // 15750 minor units with scale 3 = 15.750
      expect(body).toContain("15.750");
    });
  });

  describe("Backend Color & Zero Synthetic Sign Prefix", () => {
    it("applies backend color via inline style when provided", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 50000 as MinorUnits,
          currency: usdCurrency,
          color: "#10b981",
        },
      });
      expect(body).toContain('style="color: #10b981;"');
    });

    it("does not render style attribute when color is omitted", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 50000 as MinorUnits,
          currency: usdCurrency,
        },
      });
      expect(body).not.toContain("style=");
    });

    it("renders amount without synthetic +, -, or ↔ directional symbols", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 3200 as MinorUnits,
          currency: usdCurrency,
          color: "#f43f5e",
        },
      });
      expect(body).toMatch(/>\$32\.00</);
    });

    it("appends custom class names", () => {
      const { body } = render(AmountDisplay, {
        props: {
          amount: 10000 as MinorUnits,
          currency: usdCurrency,
          class: "font-black tracking-wide",
        },
      });
      expect(body).toContain("font-black tracking-wide");
    });
  });
});
