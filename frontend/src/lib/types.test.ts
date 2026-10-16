import { describe, expect, it } from "vitest";
import {
  err,
  expectPresent,
  InvariantViolationError,
  ok,
  toIntId,
  MinorUnits,
  toStringId,
  getTextFormValue,
  TypeConversionError,
  type AsyncState,
  type Brand,
  CategoryId,
  createEnum,
  ParseEnumError,
  UserId,
} from "./types";

describe("Frontend Core Types & Primitives (Tier 1)", () => {
  describe("TypeConversionError", () => {
    it("extends Error and formats properties correctly", () => {
      const error = new TypeConversionError("CategoryId", -5);
      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(TypeConversionError);
      expect(error.name).toBe("TypeConversionError");
      expect(error.targetType).toBe("CategoryId");
      expect(error.raw).toBe(-5);
      expect(error.message).toBe(
        "Type: CategoryId. Value: -5. Value cannot be converted to CategoryId.",
      );
    });
  });

  describe("toIntId", () => {
    it("validates and brands valid positive integers", () => {
      const id = toIntId("FamilyId", 42);
      expect(id).toBe(42);
    });

    it("throws TypeConversionError on zero and negative integers", () => {
      expect(() => toIntId("FamilyId", 0)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", -1)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", -500)).toThrow(TypeConversionError);

      try {
        toIntId("FamilyId", 0);
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(TypeConversionError);
        const err = e as TypeConversionError;
        expect(err.name).toBe("TypeConversionError");
        expect(err.targetType).toBe("FamilyId");
        expect(err.raw).toBe(0);
        expect(err.message).toBe(
          "Type: FamilyId. Value: 0. Value cannot be converted to FamilyId.",
        );
      }
    });

    it("throws TypeConversionError on floating-point numbers", () => {
      expect(() => toIntId("FamilyId", 12.34)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", 0.1)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", NaN)).toThrow(TypeConversionError);
    });

    it("throws TypeConversionError on non-number types", () => {
      expect(() => toIntId("FamilyId", "1000")).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", null)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", undefined)).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", {})).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", [])).toThrow(TypeConversionError);
      expect(() => toIntId("FamilyId", true)).toThrow(TypeConversionError);
    });
  });

  describe("toStringId", () => {
    it("validates and brands non-empty strings", () => {
      const id = toStringId("TransactionId", "tx_101");
      expect(id).toBe("tx_101");
    });

    it("trims surrounding whitespace on strings", () => {
      expect(toStringId("TransactionId", "  tx_101  ")).toBe("tx_101");
    });

    it("throws TypeConversionError on empty and whitespace-only strings", () => {
      expect(() => toStringId("TransactionId", "")).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", "   ")).toThrow(TypeConversionError);

      try {
        toStringId("TransactionId", "");
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(TypeConversionError);
        const err = e as TypeConversionError;
        expect(err.name).toBe("TypeConversionError");
        expect(err.targetType).toBe("TransactionId");
        expect(err.raw).toBe("");
        expect(err.message).toBe(
          "Type: TransactionId. Value: . Value cannot be converted to TransactionId.",
        );
      }
    });

    it("throws TypeConversionError on numbers, floats, and non-string types", () => {
      expect(() => toStringId("TransactionId", 1)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", 100)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", 0)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", -1)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", 1.5)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", null)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", undefined)).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", {})).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", [])).toThrow(TypeConversionError);
      expect(() => toStringId("TransactionId", false)).toThrow(TypeConversionError);
    });
  });

  describe("getTextFormValue", () => {
    it("extracts and trims string values from FormData", () => {
      const fd = new FormData();
      fd.append("name", "  Savings  ");
      expect(getTextFormValue(fd, "name")).toBe("Savings");
    });

    it("returns empty string when key is missing or empty", () => {
      const fd = new FormData();
      expect(getTextFormValue(fd, "missing")).toBe("");

      fd.append("empty", "   ");
      expect(getTextFormValue(fd, "empty")).toBe("");
    });
  });

  describe("MinorUnits", () => {
    it("validates and brands valid integer minor units", () => {
      const units: MinorUnits = MinorUnits(15000);
      expect(units).toBe(15000);
    });

    it("accepts zero and negative integers", () => {
      expect(MinorUnits(0)).toBe(0);
      expect(MinorUnits(-500)).toBe(-500);
    });

    it("throws TypeConversionError on floating-point numbers", () => {
      expect(() => MinorUnits(12.34)).toThrow(TypeConversionError);
    });

    it("throws TypeConversionError on non-number types", () => {
      expect(() => MinorUnits("1000")).toThrow(TypeConversionError);
      expect(() => MinorUnits(null)).toThrow(TypeConversionError);
      expect(() => MinorUnits(undefined)).toThrow(TypeConversionError);
      expect(() => MinorUnits({})).toThrow(TypeConversionError);
    });
  });

  describe("Brand nominal typing", () => {
    it("preserves underlying primitive value while distinguishing types at compile time", () => {
      const userId = UserId("usr_123");
      const categoryId = CategoryId(456);

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

  describe("createEnum", () => {
    const TestRole = createEnum("TestRole", {
      Admin: "admin",
      Viewer: "viewer",
    });

    it("exposes dot-syntax variants and values list", () => {
      expect(TestRole.Admin).toBe("admin");
      expect(TestRole.Viewer).toBe("viewer");
      expect(TestRole.$name).toBe("TestRole");
      expect(TestRole.values).toEqual(["admin", "viewer"]);
      expect(Object.isFrozen(TestRole)).toBe(true);
      expect(Object.isFrozen(TestRole.values)).toBe(true);
    });

    it("parses valid raw value correctly via .parse()", () => {
      const val = TestRole.parse("admin");
      expect(val).toBe(TestRole.Admin);
    });

    it("throws ParseEnumError on invalid value via .parse()", () => {
      expect(() => TestRole.parse("superadmin")).toThrow(ParseEnumError);
      try {
        TestRole.parse("superadmin");
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(ParseEnumError);
        const err = e as ParseEnumError;
        expect(err.enumName).toBe("TestRole");
        expect(err.allowedVariants).toEqual(["admin", "viewer"]);
        expect(err.receivedValue).toBe("superadmin");
      }
    });
  });
});
