import { describe, expect, it } from "vitest";
import {
  err,
  expectPresent,
  InvariantViolationError,
  ok,
  toMinorUnits,
  toTransactionId,
  toTypeId,
  type AsyncState,
  type Brand,
  type CategoryId,
  type MinorUnits,
  type UserId,
} from "./types";
import { ContractViolationError } from "./api/contracts";

describe("Frontend Core Types & Primitives (Tier 1)", () => {
  describe("toMinorUnits", () => {
    it("validates and brands valid integer minor units", () => {
      const units: MinorUnits = toMinorUnits(15000);
      expect(units).toBe(15000);
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

  describe("toTransactionId", () => {
    it("validates and brands string transaction IDs", () => {
      expect(toTransactionId("tx_101")).toBe("tx_101");
      expect(toTransactionId("b6c934f0-1234-4567-89ab-cdef01234567")).toBe(
        "b6c934f0-1234-4567-89ab-cdef01234567",
      );
    });

    it("converts positive integer IDs to string for backwards compatibility", () => {
      expect(toTransactionId(1)).toBe("1");
      expect(toTransactionId(100)).toBe("100");
    });

    it("throws ContractViolationError on empty strings, 0, or negative numbers", () => {
      expect(() => toTransactionId("")).toThrow(ContractViolationError);
      expect(() => toTransactionId("   ")).toThrow(ContractViolationError);
      expect(() => toTransactionId(0)).toThrow(ContractViolationError);
      expect(() => toTransactionId(-1)).toThrow(ContractViolationError);
    });

    it("throws ContractViolationError on floats, objects, null, and undefined", () => {
      expect(() => toTransactionId(1.5)).toThrow(ContractViolationError);
      expect(() => toTransactionId(null)).toThrow(ContractViolationError);
      expect(() => toTransactionId(undefined)).toThrow(ContractViolationError);
      expect(() => toTransactionId({})).toThrow(ContractViolationError);
    });
  });

  describe("toTypeId", () => {
    it("validates and brands positive integer type IDs", () => {
      expect(toTypeId(1)).toBe(1);
      expect(toTypeId(50)).toBe(50);
    });

    it("throws ContractViolationError on 0 or negative numbers", () => {
      expect(() => toTypeId(0)).toThrow(ContractViolationError);
      expect(() => toTypeId(-5)).toThrow(ContractViolationError);
    });

    it("throws ContractViolationError on floats and non-numbers", () => {
      expect(() => toTypeId(2.5)).toThrow(ContractViolationError);
      expect(() => toTypeId("50")).toThrow(ContractViolationError);
      expect(() => toTypeId(undefined)).toThrow(ContractViolationError);
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
      if (res.ok === true) {
        expect(res.value).toBe(42);
      }
    });

    it("creates err result", () => {
      const res = err("something broke");
      expect(res.ok).toBe(false);
      if (res.ok === false) {
        expect(res.error).toBe("something broke");
      }
    });
  });

  describe("expectPresent & InvariantViolationError", () => {
    it("returns non-nullable value when present", () => {
      const val = expectPresent("hello", "TEST.PRESENT", "Should be present");
      expect(val).toBe("hello");

      const num = expectPresent(0, "TEST.ZERO", "Zero is valid");
      expect(num).toBe(0);

      const bool = expectPresent(false, "TEST.BOOL", "False is valid");
      expect(bool).toBe(false);
    });

    it("throws InvariantViolationError with unique screaming action token on null or undefined", () => {
      expect(() => expectPresent(null, "TEST.ACTION.NULL_VAL", "Value was null")).toThrow(
        InvariantViolationError,
      );

      try {
        expectPresent(undefined, "TEST.ACTION.MISSING", "Value was undefined");
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(InvariantViolationError);
        const err = e as InvariantViolationError;
        expect(err.action).toBe("TEST.ACTION.MISSING");
        expect(err.message).toBe("[TEST.ACTION.MISSING] InvariantViolation: Value was undefined");
      }
    });
  });
});
