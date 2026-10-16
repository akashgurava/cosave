import { describe, expect, it } from "vitest";
import { z } from "zod";
import {
  IdCreationError,
  InvariantViolationError,
  ParseEnumError,
  ParseModelError,
} from "./errors";
import {
  err,
  expectPresent,
  ok,
  toIntId,
  MinorUnits,
  toStringId,
  getTextFormValue,
  type AsyncState,
  type Brand,
  CategoryId,
  createEnum,
  deepFreeze,
  defineModel,
  UserId,
} from "./types";

describe("Frontend Core Types & Primitives (Tier 1)", () => {
  describe("toIntId", () => {
    it("validates and brands valid positive integers", () => {
      const id = toIntId("FamilyId", 42);
      expect(id).toBe(42);
    });

    it("throws IdCreationError on zero and negative integers", () => {
      expect(() => toIntId("FamilyId", 0)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", -1)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", -500)).toThrow(IdCreationError);

      try {
        toIntId("FamilyId", 0);
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(IdCreationError);
        const err = e as IdCreationError;
        expect(err.name).toBe("IdCreationError");
        expect(err.targetType).toBe("number");
        expect(err.idName).toBe("FamilyId");
        expect(err.raw).toBe(0);
        expect(err.action).toBe("UI.ID_CREATION.TYPE_CONVERSION.FamilyId");
        expect(err.message).toBe(
          "Type: number. ID Name: FamilyId Value: 0. Failed to create ID.",
        );
      }
    });

    it("throws IdCreationError on floating-point numbers", () => {
      expect(() => toIntId("FamilyId", 12.34)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", 0.1)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", NaN)).toThrow(IdCreationError);
    });

    it("throws IdCreationError on non-number types", () => {
      expect(() => toIntId("FamilyId", "1000")).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", null)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", undefined)).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", {})).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", [])).toThrow(IdCreationError);
      expect(() => toIntId("FamilyId", true)).toThrow(IdCreationError);
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

    it("throws IdCreationError on empty and whitespace-only strings", () => {
      expect(() => toStringId("TransactionId", "")).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", "   ")).toThrow(IdCreationError);

      try {
        toStringId("TransactionId", "");
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(IdCreationError);
        const err = e as IdCreationError;
        expect(err.name).toBe("IdCreationError");
        expect(err.targetType).toBe("string");
        expect(err.idName).toBe("TransactionId");
        expect(err.raw).toBe("");
        expect(err.action).toBe("UI.ID_CREATION.TYPE_CONVERSION.TransactionId");
        expect(err.message).toBe(
          "Type: string. ID Name: TransactionId Value: . Failed to create ID.",
        );
      }
    });

    it("throws IdCreationError on numbers, floats, and non-string types", () => {
      expect(() => toStringId("TransactionId", 1)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", 100)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", 0)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", -1)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", 1.5)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", null)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", undefined)).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", {})).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", [])).toThrow(IdCreationError);
      expect(() => toStringId("TransactionId", false)).toThrow(IdCreationError);
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

    it("throws IdCreationError on floating-point numbers", () => {
      expect(() => MinorUnits(12.34)).toThrow(IdCreationError);
    });

    it("throws IdCreationError on non-number types", () => {
      expect(() => MinorUnits("1000")).toThrow(IdCreationError);
      expect(() => MinorUnits(null)).toThrow(IdCreationError);
      expect(() => MinorUnits(undefined)).toThrow(IdCreationError);
      expect(() => MinorUnits({})).toThrow(IdCreationError);
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

    it("provides an embedded Zod schema for composition", () => {
      expect(TestRole.schema).toBeDefined();
      expect(TestRole.schema.safeParse("admin").success).toBe(true);
      expect(TestRole.schema.safeParse("superadmin").success).toBe(false);
    });
  });

  describe("deepFreeze", () => {
    it("recursively freezes nested objects and arrays", () => {
      const data = {
        name: "CoSave",
        nested: { count: 10 },
        list: [{ id: 1 }, { id: 2 }],
      };
      const frozen = deepFreeze(data);
      expect(Object.isFrozen(frozen)).toBe(true);
      expect(Object.isFrozen(frozen.nested)).toBe(true);
      expect(Object.isFrozen(frozen.list)).toBe(true);
      expect(Object.isFrozen(frozen.list[0])).toBe(true);
    });

    it("handles primitives, null, and already frozen objects gracefully", () => {
      expect(deepFreeze(null)).toBeNull();
      expect(deepFreeze(42)).toBe(42);
      expect(deepFreeze("str")).toBe("str");
    });
  });

  describe("parseModel & defineModel", () => {
    const SimpleUser = defineModel(
      "SimpleUser",
      z.object({
        name: z.string().min(1),
        age: z.number().int().positive(),
        role: z.enum(["admin", "user"]),
      }),
    );

    it("parses valid data and returns deep-frozen result", () => {
      const valid = SimpleUser.parse({ name: "Alice", age: 30, role: "admin" });
      expect(valid).toEqual({ name: "Alice", age: 30, role: "admin" });
      expect(Object.isFrozen(valid)).toBe(true);
    });

    it("throws ParseModelError on non-object payload", () => {
      try {
        SimpleUser.parse("not an object");
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(ParseModelError);
        const err = e as ParseModelError;
        expect(err.modelName).toBe("SimpleUser");
        expect(err.issues[0]?.code.isInvalidType()).toBe(true);
      }
    });

    it("captures multiple issues with normalized ModelIssue codes", () => {
      try {
        SimpleUser.parse({ age: -5, role: "superadmin" });
        expect.unreachable();
      } catch (e) {
        expect(e).toBeInstanceOf(ParseModelError);
        const err = e as ParseModelError;
        expect(err.modelName).toBe("SimpleUser");
        expect(err.issues).toHaveLength(3); // missing name, invalid age (too_small), invalid enum role

        const missingName = err.issues.find((i) => i.path === "name");
        expect(missingName).toBeDefined();
        expect(missingName?.code.isMissingField()).toBe(true);

        const invalidAge = err.issues.find((i) => i.path === "age");
        expect(invalidAge).toBeDefined();
        expect(invalidAge?.code.isConstraintViolation()).toBe(true);

        const invalidRole = err.issues.find((i) => i.path === "role");
        expect(invalidRole).toBeDefined();
        expect(invalidRole?.code.isInvalidEnum()).toBe(true);
      }
    });
  });
});
