import { describe, it, expect, vi } from "vitest";
import { ApiError } from "$lib/api/contracts";
import { InvariantViolationError } from "$lib/types";
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

  it("formats standard Error", () => {
    const err = new Error("Network timeout");
    const formatted = formatErrorMessage(err);
    expect(formatted.title).toBe("Unexpected Error");
    expect(formatted.message).toBe("Network timeout");
    expect(formatted.action).toBeUndefined();
  });

  it("formats unknown string error", () => {
    const formatted = formatErrorMessage("Something broke");
    expect(formatted.title).toBe("Unexpected Error");
    expect(formatted.message).toBe("Something broke");
  });

  it("logs to console.error when errorToToast is invoked", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const err = new Error("Test error for toast");

    errorToToast(err);

    expect(consoleSpy).toHaveBeenCalledWith("[cosave:error]", err);
    consoleSpy.mockRestore();
  });
});
