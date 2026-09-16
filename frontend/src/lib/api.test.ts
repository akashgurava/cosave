import { describe, it, expect } from "vitest";
import {
  Code,
  Status,
  extractApiResponse,
  extractData,
  extractCode,
  extractStatus,
  UnanticipatedCodeError,
  UnanticipatedStatusError,
  ApiError,
  parseCode,
  parseStatus,
} from "./api";

describe("API Response Utilities", () => {
  it("extracts valid response correctly", () => {
    const raw = {
      code: 0,
      status: "HEALTHY",
      data: { service: "cosave" },
    };

    const parsed = extractApiResponse<{ service: string }>(raw);
    expect(parsed.code).toBe(Code.Zero);
    expect(parsed.status).toBe(Status.Healthy);
    expect(parsed.data.service).toBe("cosave");

    expect(extractCode(raw)).toBe(Code.Zero);
    expect(extractStatus(raw)).toBe(Status.Healthy);
    expect(extractData<{ service: string }>(raw)).toEqual({ service: "cosave" });
  });

  it("builders return matching values", () => {
    expect(Code.zero()).toBe(Code.Zero);
    expect(Code.badRequest()).toBe(Code.BadRequest);
    expect(Code.unauthorized()).toBe(Code.Unauthorized);
    expect(Code.conflict()).toBe(Code.Conflict);
    expect(Code.internalError()).toBe(Code.InternalError);

    expect(Status.healthy()).toBe(Status.Healthy);
    expect(Status.ok()).toBe(Status.Ok);
    expect(Status.badRequest()).toBe(Status.BadRequest);
    expect(Status.unauthenticated()).toBe(Status.Unauthenticated);
    expect(Status.invalidCredentials()).toBe(Status.InvalidCredentials);
    expect(Status.userExists()).toBe(Status.UserExists);
    expect(Status.internalError()).toBe(Status.InternalError);
  });

  it("parses auth error codes and statuses correctly", () => {
    expect(parseCode(400)).toBe(Code.BadRequest);
    expect(parseCode(401)).toBe(Code.Unauthorized);
    expect(parseCode(409)).toBe(Code.Conflict);

    expect(parseStatus("UNAUTHENTICATED")).toBe(Status.Unauthenticated);
    expect(parseStatus("INVALID_CREDENTIALS")).toBe(Status.InvalidCredentials);
    expect(parseStatus("USER_EXISTS")).toBe(Status.UserExists);
  });

  it("throws UnanticipatedCodeError on unexpected code", () => {
    const invalid = {
      code: 999,
      status: "HEALTHY",
      data: {},
    };

    expect(() => extractApiResponse(invalid)).toThrow(UnanticipatedCodeError);
  });

  it("throws UnanticipatedStatusError on unexpected status", () => {
    const invalid = {
      code: 0,
      status: "SOME_UNKNOWN_STATUS",
      data: {},
    };

    expect(() => extractApiResponse(invalid)).toThrow(UnanticipatedStatusError);
  });

  it("throws ApiError when response is not an object", () => {
    expect(() => extractApiResponse(null)).toThrow(ApiError);
    expect(() => extractApiResponse("bad-response")).toThrow(ApiError);
  });
});
