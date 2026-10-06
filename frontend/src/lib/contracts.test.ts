import { describe, it, expect } from "vitest";
import {
  Code,
  Status,
  parseCode,
  parseStatus,
  parseNull,
  isObject,
  ApiError,
  ContractViolationError,
} from "./contracts";

describe("Contracts & Runtime Envelopes", () => {
  describe("parseCode", () => {
    it("parses known backend codes", () => {
      expect(parseCode(0)).toBe(Code.Zero);
      expect(parseCode(400)).toBe(Code.BadRequest);
      expect(parseCode(401)).toBe(Code.Unauthorized);
      expect(parseCode(409)).toBe(Code.Conflict);
      expect(parseCode(500)).toBe(Code.InternalError);
    });

    it("throws ApiError on unexpected code", () => {
      expect(() => parseCode(999)).toThrow(ApiError);
      expect(() => parseCode("unknown")).toThrow(ApiError);
    });
  });

  describe("parseStatus", () => {
    it("parses known backend statuses", () => {
      expect(parseStatus("HEALTHY")).toBe(Status.Healthy);
      expect(parseStatus("OK")).toBe(Status.Ok);
      expect(parseStatus("BAD_REQUEST")).toBe(Status.BadRequest);
      expect(parseStatus("UNAUTHENTICATED")).toBe(Status.Unauthenticated);
      expect(parseStatus("INVALID_CREDENTIALS")).toBe(Status.InvalidCredentials);
      expect(parseStatus("USER_ALREADY_EXISTS")).toBe(Status.UserAlreadyExists);
      expect(parseStatus("INTERNAL_ERROR")).toBe(Status.InternalError);
    });

    it("throws ApiError on unexpected status", () => {
      expect(() => parseStatus("UNKNOWN_STATUS")).toThrow(ApiError);
      expect(() => parseStatus(123)).toThrow(ApiError);
    });
  });

  describe("parseNull", () => {
    it("returns null for null and undefined", () => {
      expect(parseNull(null)).toBeNull();
      expect(parseNull(undefined)).toBeNull();
    });

    it("throws ContractViolationError for non-null data", () => {
      expect(() => parseNull({ key: "val" })).toThrow(ContractViolationError);
      expect(() => parseNull("string")).toThrow(ContractViolationError);
      expect(() => parseNull(0)).toThrow(ContractViolationError);
    });
  });

  describe("isObject", () => {
    it("identifies plain objects correctly", () => {
      expect(isObject({})).toBe(true);
      expect(isObject({ a: 1 })).toBe(true);
      expect(isObject(null)).toBe(false);
      expect(isObject(undefined)).toBe(false);
      expect(isObject([])).toBe(false);
      expect(isObject("string")).toBe(false);
      expect(isObject(123)).toBe(false);
    });
  });
});
