import { describe, expect, it } from "vitest";
import {
  err,
  ok,
  toAmountCents,
  toMinorUnits,
  type AsyncState,
  type Brand,
  type CategoryId,
  type MinorUnits,
  type UserId,
} from "./core";
import { ContractViolationError } from "$lib/api";

describe("Frontend Core Types & Primitives (Tier 1)", () => {
  describe("toMinorUnits & toAmountCents", () => {
    it("validates and brands valid integer minor units", () => {
      const units: MinorUnits = toMinorUnits(15000);
      expect(units).toBe(15000);
      const cents = toAmountCents(15000);
      expect(cents).toBe(15000);
    });

    it("accepts zero and negative integers", () => {
      expect(toMinorUnits(0)).toBe(0);
      expect(toMinorUnits(-500)).toBe(-500);
    });

    it("throws ContractViolationError on floating-point numbers", () => {
      expect(() => toMinorUnits(12.34)).toThrow(ContractViolationError);
    });

    it("throws ContractViolationError on non-number types", () => {
      expect(() => toMinorUnits("1000")).toThrow(ContractViolationError);
      expect(() => toMinorUnits(null)).toThrow(ContractViolationError);
      expect(() => toMinorUnits(undefined)).toThrow(ContractViolationError);
      expect(() => toMinorUnits({})).toThrow(ContractViolationError);
    });
  });

  describe("Brand nominal typing", () => {
    it("preserves underlying primitive value while distinguishing types at compile time", () => {
      const userId = "usr_123" as UserId;
      const categoryId = 456 as CategoryId;

      expect(typeof userId).toBe("string");
      expect(typeof categoryId).toBe("number");
      expect(userId).toBe("usr_123");
      expect(categoryId).toBe(456);

      // Custom brand test
      type OrderId = Brand<string, "OrderId">;
      const orderId = "ord_999" as OrderId;
      expect(orderId).toBe("ord_999");
    });
  });

  describe("AsyncState discriminated union", () => {
    it("exhaustively narrows each status variant", () => {
      const idleState: AsyncState<number> = { status: "idle" };
      const loadingState: AsyncState<number> = { status: "loading" };
      const successState: AsyncState<number> = { status: "success", data: 100 };
      const errorState: AsyncState<number> = {
        status: "error",
        error: { action: "TEST.FAIL", message: "Failed" },
      };

      function renderStatus(state: AsyncState<number>): string {
        switch (state.status) {
          case "idle":
            return "IDLE";
          case "loading":
            return "LOADING";
          case "success":
            return `VALUE: ${state.data}`;
          case "error":
            return `ERROR: ${state.error.message}`;
        }
      }

      expect(renderStatus(idleState)).toBe("IDLE");
      expect(renderStatus(loadingState)).toBe("LOADING");
      expect(renderStatus(successState)).toBe("VALUE: 100");
      expect(renderStatus(errorState)).toBe("ERROR: Failed");
    });
  });

  describe("Result", () => {
    it("creates ok result", () => {
      const res = ok(42);
      expect(res.ok).toBe(true);
      if (res.ok) {
        expect(res.value).toBe(42);
      }
    });

    it("creates err result", () => {
      const res = err("something broke");
      expect(res.ok).toBe(false);
      if (!res.ok) {
        expect(res.error).toBe("something broke");
      }
    });
  });
});
