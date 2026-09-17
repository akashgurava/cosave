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
  ContractViolationError,
  ApiError,
  parseCode,
  parseStatus,
  parseRole,
  parseUserDto,
  parseNullableUserDto,
  parseNull,
  parseTransactionTypeItem,
  parseSubcategoryItem,
  parseCategoryItem,
  parseCategoryHierarchyResponse,
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

describe("Strict Type Parsers & Contract Enforcement", () => {
  it("parses valid UserDto without inferences", () => {
    const raw = {
      id: "usr-123",
      name: "alice",
      role: "admin",
      created_at: 1700000000,
    };

    const user = parseUserDto(raw);
    expect(user.id).toBe("usr-123");
    expect(user.name).toBe("alice");
    expect(user.role).toBe("admin");
    expect(user.created_at).toBe(1700000000);
  });

  it("throws ContractViolationError on invalid UserDto fields", () => {
    expect(() => parseUserDto(null)).toThrow(ContractViolationError);
    expect(() => parseUserDto({ id: 123, name: "alice", role: "member", created_at: 1 })).toThrow(
      ContractViolationError,
    );
    expect(() =>
      parseUserDto({ id: "1", name: "alice", role: "superadmin", created_at: 1 }),
    ).toThrow(ContractViolationError);
    expect(() => parseRole("invalid_role")).toThrow(ContractViolationError);
  });

  it("parses nullable user DTO and null responses correctly", () => {
    expect(parseNullableUserDto(null)).toBeNull();
    expect(parseNull(null)).toBeNull();
    expect(() => parseNull({ some: "data" })).toThrow(ContractViolationError);
  });

  it("parses valid CategoryHierarchyResponse strictly", () => {
    const raw = {
      types: [
        { id: "type-income", name: "Income", color: "#10b981" },
        { id: "type-expense", name: "Expense", color: "#f43f5e" },
      ],
      categories: [
        {
          id: "cat-salary",
          name: "Salary",
          type: "Income",
          subcategories: [{ id: "sub-primary", name: "Primary Employer" }],
        },
      ],
    };

    const parsed = parseCategoryHierarchyResponse(raw);
    expect(parsed.types).toHaveLength(2);
    expect(parsed.types[0]).toEqual({
      id: "type-income",
      name: "Income",
      color: "#10b981",
    });
    expect(parsed.categories).toHaveLength(1);
    expect(parsed.categories[0].subcategories).toHaveLength(1);
    expect(parsed.categories[0].subcategories[0]).toEqual({
      id: "sub-primary",
      name: "Primary Employer",
    });
  });

  it("rejects malformed hierarchy responses", () => {
    expect(() => parseCategoryHierarchyResponse(null)).toThrow(ContractViolationError);
    expect(() => parseCategoryHierarchyResponse({ types: "not-an-array", categories: [] })).toThrow(
      ContractViolationError,
    );
    expect(() => parseTransactionTypeItem({ id: "1", name: 123, color: "#fff" })).toThrow(
      ContractViolationError,
    );
    expect(() => parseSubcategoryItem({ id: "1" })).toThrow(ContractViolationError);
    expect(() =>
      parseCategoryItem({
        id: "cat-1",
        name: "Food",
        type: "Expense",
        subcategories: "not-an-array",
      }),
    ).toThrow(ContractViolationError);
  });

  it("extractApiResponse enforces schema validation when parser is supplied", () => {
    const raw = {
      code: 0,
      status: "OK",
      data: {
        id: "type-test",
        name: "Test Type",
        color: "#3b82f6",
      },
    };

    const response = extractApiResponse(raw, parseTransactionTypeItem);
    expect(response.data.id).toBe("type-test");
    expect(response.data.name).toBe("Test Type");

    const malformedRaw = {
      code: 0,
      status: "OK",
      data: {
        id: "type-test",
        name: 123, // wrong type
        color: "#3b82f6",
      },
    };

    expect(() => extractApiResponse(malformedRaw, parseTransactionTypeItem)).toThrow(
      ContractViolationError,
    );
  });
});
