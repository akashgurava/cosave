import { describe, expect, it } from "vitest";
import { z } from "zod";
import { IdCreationError, InvariantViolationError, normalizeZodIssue, ParseModelError } from "./errors";

describe("normalizeZodIssue", () => {
  it("differentiates missing fields from type mismatches using raw payload inspection", () => {
    const userSchema = z.object({ age: z.number() });

    const rawMissing = {};
    const resMissing = userSchema.safeParse(rawMissing);
    expect(resMissing.success).toBe(false);
    if (!resMissing.success) {
      const result = normalizeZodIssue(resMissing.error.issues[0]!, rawMissing, "User");
      expect(result.path).toBe("age");
      expect(result.code.isMissingField()).toBe(true);
      expect(result.code.isInvalidType()).toBe(false);
    }

    const rawTypeMismatch = { age: "not-a-number" };
    const resMismatch = userSchema.safeParse(rawTypeMismatch);
    expect(resMismatch.success).toBe(false);
    if (!resMismatch.success) {
      const result = normalizeZodIssue(resMismatch.error.issues[0]!, rawTypeMismatch, "User");
      expect(result.path).toBe("age");
      expect(result.code.isInvalidType()).toBe(true);
      expect(result.code.isMissingField()).toBe(false);
    }
  });

  it("accurately navigates deeply nested paths and array indices", () => {
    const familySchema = z.object({
      family: z.object({
        members: z.array(z.object({ name: z.string(), role: z.string() })),
      }),
    });

    const rawMissing = {
      family: {
        members: [{ name: "Alice" }],
      },
    };
    const resMissing = familySchema.safeParse(rawMissing);
    expect(resMissing.success).toBe(false);
    if (!resMissing.success) {
      const result = normalizeZodIssue(resMissing.error.issues[0]!, rawMissing, "Family");
      expect(result.path).toBe("family.members.0.role");
      expect(result.code.isMissingField()).toBe(true);
    }

    const rawType = {
      family: {
        members: [{ name: 123, role: "parent" }],
      },
    };
    const resType = familySchema.safeParse(rawType);
    expect(resType.success).toBe(false);
    if (!resType.success) {
      const result = normalizeZodIssue(resType.error.issues[0]!, rawType, "Family");
      expect(result.path).toBe("family.members.0.name");
      expect(result.code.isInvalidType()).toBe(true);
    }
  });

  it("classifies enum and constraint violations", () => {
    const accountSchema = z.object({
      status: z.enum(["active", "inactive"]),
      count: z.number().nonnegative(),
    });

    const raw = { status: "unknown", count: -1 };
    const res = accountSchema.safeParse(raw);
    expect(res.success).toBe(false);
    if (!res.success) {
      const statusIssue = res.error.issues.find((i) => i.path[0] === "status");
      expect(statusIssue).toBeDefined();
      expect(normalizeZodIssue(statusIssue!, raw, "Account").code.isInvalidEnum()).toBe(true);

      const countIssue = res.error.issues.find((i) => i.path[0] === "count");
      expect(countIssue).toBeDefined();
      expect(normalizeZodIssue(countIssue!, raw, "Account").code.isConstraintViolation()).toBe(
        true,
      );
    }
  });

  it("falls back to modelName when issue path is empty", () => {
    const objSchema = z.object({ name: z.string() });
    const raw = "plain string";
    const res = objSchema.safeParse(raw);
    expect(res.success).toBe(false);
    if (!res.success) {
      const result = normalizeZodIssue(res.error.issues[0]!, raw, "Family");
      expect(result.path).toBe("Family");
      expect(result.code.isInvalidType()).toBe(true);
    }
  });
});

describe("IdCreationError", () => {
  it("extends Error and formats properties correctly", () => {
    const error = new IdCreationError("number", "CategoryId", -5);
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(IdCreationError);
    expect(error.name).toBe("IdCreationError");
    expect(error.targetType).toBe("number");
    expect(error.idName).toBe("CategoryId");
    expect(error.raw).toBe(-5);
    expect(error.action).toBe("UI.ID_CREATION.TYPE_CONVERSION.CategoryId");
    expect(error.message).toBe("Type: number. ID Name: CategoryId Value: -5. Failed to create ID.");
  });
});

describe("InvariantViolationError", () => {
  it("extends Error and formats properties correctly", () => {
    const error = new InvariantViolationError(
      "TEST.INVARIANT.CHECK",
      "Expected entity to be present",
    );
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(InvariantViolationError);
    expect(error.name).toBe("InvariantViolationError");
    expect(error.action).toBe("TEST.INVARIANT.CHECK");
    expect(error.message).toBe("[TEST.INVARIANT.CHECK] InvariantViolation: Expected entity to be present");
  });
});
