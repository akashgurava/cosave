import { describe, it, expect, vi } from "vitest";
import { ApiError } from "$lib/api/contracts";
import {
  IdCreationError,
  InvariantViolationError,
  ParseEnumError,
  ParseModelError,
} from "$lib/errors";
import { formatErrorMessage, errorToToast } from "./toast";

describe("errorToToast & formatErrorMessage", () => {
  it("formats ApiError into title, message, and action", () => {
    const apiErr = new ApiError(
      "Email already exists",
      409,
      409,
      "USER_ALREADY_EXISTS",
      null,
      "AUTH.REGISTER.DUPLICATE",
    );

    const formatted = formatErrorMessage(apiErr);
    expect(formatted.title).toBe("User Already Exists");
    expect(formatted.message).toBe("Email already exists");
    expect(formatted.action).toBe("AUTH.REGISTER.DUPLICATE");
  });

  it("formats InvariantViolationError with action token", () => {
    const invErr = new InvariantViolationError(
      "VIEW.TRANSACTION_ROW.RESOLVE_MEMBER",
      "Member 1 not found",
    );

    const formatted = formatErrorMessage(invErr);
    expect(formatted.title).toBe("State Invariant Violation");
    expect(formatted.message).toContain("Member 1 not found");
    expect(formatted.action).toBe("VIEW.TRANSACTION_ROW.RESOLVE_MEMBER");
  });

  it("formats IdCreationError into title, message, and action token", () => {
    const err = new IdCreationError("number", "CategoryId", -1);
    const formatted = formatErrorMessage(err);
    expect(formatted.title).toBe("ID Creation Error");
    expect(formatted.message).toBe(
      "Type: number. ID Name: CategoryId Value: -1. Failed to create ID.",
    );
    expect(formatted.action).toBe("UI.ID_CREATION.TYPE_CONVERSION.CategoryId");
  });

  it("formats ParseModelError into title, message, and null action", () => {
    const err = new ParseModelError("User", [], {});
    const formatted = formatErrorMessage(err);
    expect(formatted.title).toBe("Validation Error");
    expect(formatted.message).toBe("[User] Validation failed: ");
    expect(formatted.action).toBeNull();
  });

  it("formats ParseEnumError into title, message, and null action", () => {
    const err = new ParseEnumError("Role", ["admin", "member"], "guest");
    const formatted = formatErrorMessage(err);
    expect(formatted.title).toBe("Invalid Enum Value");
    expect(formatted.message).toContain("ParseEnumError. ENUM Name: Role.");
    expect(formatted.action).toBeNull();
  });

  it("formats standard Error with null action", () => {
    const err = new Error("Network timeout");
    const formatted = formatErrorMessage(err);
    expect(formatted.title).toBe("Unexpected Error");
    expect(formatted.message).toBe("Network timeout");
    expect(formatted.action).toBeNull();
  });

  it("formats unknown string error with null action", () => {
    const formatted = formatErrorMessage("Something broke");
    expect(formatted.title).toBe("Unexpected Error");
    expect(formatted.message).toBe("Something broke");
    expect(formatted.action).toBeNull();
  });

  it("logs to console.error when errorToToast is invoked", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const err = new Error("Test error for toast");

    errorToToast(err);

    expect(consoleSpy).toHaveBeenCalledWith("[cosave:error]", err);
    consoleSpy.mockRestore();
  });
});
