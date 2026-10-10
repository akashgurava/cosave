import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, ContractViolationError, buildUrl, onUnauthorized } from "./client";
import { MemoryTransportAdapter } from "./testing";
import { parseAccount, parseMember } from "../features/family/types";

describe("Deepened ApiClient (Caller-Optimized REST Client)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: (() => void) | undefined;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    if (restoreTransport !== undefined) {
      restoreTransport();
    }
  });

  it("unwraps successful backend envelope and directly returns typed data", async () => {
    memoryTransport.on("GET", "/api/v1/auth/me", () => ({
      code: 0,
      status: "OK",
      data: { id: "usr-1", username: "Alice", role: "admin", createdAt: 1700000000 },
    }));

    const user = await api.get<{ id: string; username: string }>("/api/v1/auth/me");
    expect(user.id).toBe("usr-1");
    expect(user.username).toBe("Alice");
  });

  it("posts JSON body automatically and parses response data", async () => {
    memoryTransport.on("POST", "/api/v1/categories/types", (req) => {
      expect(req.headers["Content-Type"]).toBe("application/json");
      expect(req.body).toBe(JSON.stringify({ name: "Savings", color: "#f59e0b" }));
      return {
        code: 0,
        status: "OK",
        data: { id: "type-savings", name: "Savings", color: "#f59e0b" },
      };
    });

    const created = await api.post<{ id: string; name: string }>("/api/v1/categories/types", {
      name: "Savings",
      color: "#f59e0b",
    });
    expect(created.id).toBe("type-savings");
    expect(created.name).toBe("Savings");
  });

  it("interpolates path parameters and URI-encodes them automatically", async () => {
    memoryTransport.on("PATCH", "/api/v1/categories/cat%2Ffood/color", (req) => {
      expect(req.body).toBe(JSON.stringify({ color: "#10b981" }));
      return {
        code: 0,
        status: "OK",
        data: { id: "cat/food", color: "#10b981" },
      };
    });

    const res = await api.patch<{ id: string; color: string }>(
      "/api/v1/categories/:id/color",
      { color: "#10b981" },
      { pathParams: { id: "cat/food" } },
    );
    expect(res.id).toBe("cat/food");
    expect(res.color).toBe("#10b981");
  });

  it("deletes resources with interpolated path params", async () => {
    memoryTransport.on("DELETE", "/api/v1/categories/subcategories/sub-123", () => ({
      code: 0,
      status: "OK",
      data: { deleted: true },
    }));

    const res = await api.delete<{ deleted: boolean }>("/api/v1/categories/subcategories/:id", {
      pathParams: { id: "sub-123" },
    });
    expect(res.deleted).toBe(true);
  });

  it("deletes resources with optional body payload when required by backend", async () => {
    memoryTransport.on("DELETE", "/api/v1/transactions/tx-123", (req) => {
      expect(req.headers["Content-Type"]).toBe("application/json");
      expect(req.body).toBe(JSON.stringify({ source: "manual" }));
      return {
        code: 0,
        status: "OK",
        data: null,
      };
    });

    const res = await api.delete<null>("/api/v1/transactions/:id", {
      pathParams: { id: "tx-123" },
      body: { source: "manual" },
    });
    expect(res).toBeNull();
  });

  it("serializes query parameters and prunes null/undefined values", async () => {
    memoryTransport.on("GET", "/api/v1/transactions", (req) => {
      expect(req.url).toBe("/api/v1/transactions?limit=20&type=Expense&tags=groceries&tags=food");
      return {
        code: 0,
        status: "OK",
        data: [{ id: "tx-1" }],
      };
    });

    const txs = await api.get<Array<{ id: string }>>("/api/v1/transactions", {
      query: {
        limit: 20,
        type: "Expense",
        emptyField: null,
        ignoredField: undefined,
        tags: ["groceries", "food"],
      },
    });
    expect(txs).toHaveLength(1);
    const firstTx = txs[0];
    expect(firstTx).toBeDefined();
    if (firstTx !== undefined) {
      expect(firstTx.id).toBe("tx-1");
    }
  });

  it("throws normalized ApiError on 401 unauthenticated", async () => {
    memoryTransport.on("GET", "/api/v1/auth/me", () => ({
      code: 401,
      status: "UNAUTHENTICATED",
      data: null,
    }));

    let error: ApiError | null = null;
    try {
      await api.get("/api/v1/auth/me");
    } catch (err) {
      error = err as ApiError;
    }

    expect(error).toBeInstanceOf(ApiError);
    expect(error).not.toBeNull();
    if (error !== null) {
      expect(error.httpStatus).toBe(401);
      expect(error.code).toBe(401);
      expect(error.apiStatus).toBe("UNAUTHENTICATED");
    }
  });

  it("invokes onUnauthorized listener on 401 errors except login", async () => {
    let capturedError: ApiError | null = null;
    let capturedPath: string | null = null;

    const unsubscribe = onUnauthorized((err, path) => {
      capturedError = err;
      capturedPath = path;
    });

    try {
      // 1. Protected endpoint returning 401 triggers listener
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: 401,
        status: "UNAUTHORIZED",
        data: { action: "AUTH.EXTRACT_USER.INVALID_TOKEN", message: "Token expired" },
      }));

      await expect(api.get("/api/v1/config/family")).rejects.toThrow(ApiError);
      expect(capturedError).not.toBeNull();
      if (capturedError !== null) {
        expect((capturedError as ApiError).httpStatus).toBe(401);
      }
      expect(capturedPath).toBe("/api/v1/config/family");

      // Reset captured
      capturedError = null;
      capturedPath = null;

      // 2. /api/v1/auth/login returning 401 does NOT trigger listener (invalid credentials)
      memoryTransport.on("POST", "/api/v1/auth/login", () => ({
        code: 401,
        status: "INVALID_CREDENTIALS",
        data: { action: "AUTH.LOGIN.INVALID_CREDENTIALS", message: "Wrong password" },
      }));

      await expect(
        api.post("/api/v1/auth/login", { username: "u", password: "p" }),
      ).rejects.toThrow(ApiError);
      expect(capturedError).toBeNull();
      expect(capturedPath).toBeNull();
    } finally {
      unsubscribe();
    }
  });

  it("throws normalized ApiError on 409 conflict", async () => {
    memoryTransport.on("POST", "/api/v1/auth/register", () => ({
      code: 409,
      status: "USER_ALREADY_EXISTS",
      data: null,
    }));

    let error: ApiError | null = null;
    try {
      await api.post("/api/v1/auth/register", { username: "alice", password: "pwd" });
    } catch (err) {
      error = err as ApiError;
    }

    expect(error).toBeInstanceOf(ApiError);
    expect(error).not.toBeNull();
    if (error !== null) {
      expect(error.code).toBe(409);
      expect(error.apiStatus).toBe("USER_ALREADY_EXISTS");
    }
  });

  it("extracts structured ErrorPayload with action and message into ApiError", async () => {
    memoryTransport.on("POST", "/api/v1/categories/types", () => ({
      code: 409,
      status: "TYPE_ALREADY_EXISTS",
      data: {
        action: "CONFIG.CATEGORIES.CREATE_TYPE",
        message: "Transaction type 'Income' already exists.",
      },
    }));

    let error: ApiError | null = null;
    try {
      await api.post("/api/v1/categories/types", { name: "Income" });
    } catch (err) {
      error = err as ApiError;
    }

    expect(error).toBeInstanceOf(ApiError);
    expect(error).not.toBeNull();
    if (error !== null) {
      expect(error.httpStatus).toBe(409);
      expect(error.code).toBe(409);
      expect(error.apiStatus).toBe("TYPE_ALREADY_EXISTS");
      expect(error.message).toBe("Transaction type 'Income' already exists.");
      expect(error.action).toBe("CONFIG.CATEGORIES.CREATE_TYPE");
    }
  });

  it("enforces contract schema when schema validator is provided", async () => {
    memoryTransport.on("GET", "/api/v1/test", () => ({
      code: 0,
      status: "OK",
      data: { count: "invalid-string" },
    }));

    const strictNumberSchema = (raw: unknown) => {
      if (
        typeof raw === "object" &&
        raw !== null &&
        typeof (raw as { count: unknown }).count === "number"
      ) {
        return raw as { count: number };
      }
      throw new Error("count must be a number");
    };

    await expect(api.get("/api/v1/test", { schema: strictNumberSchema })).rejects.toThrow(
      ContractViolationError,
    );
  });
});

describe("URL Builder Utility", () => {
  it("interpolates path parameters and encodings", () => {
    const url = buildUrl("/api/v1/items/:id/details/{subId}", {
      id: "a/b",
      subId: "c&d",
    });
    expect(url).toBe("/api/v1/items/a%2Fb/details/c%26d");
  });

  it("builds clean query string without null or undefined", () => {
    const url = buildUrl("/api/v1/items", undefined, {
      active: true,
      count: 10,
      filter: null,
      missing: undefined,
    });
    expect(url).toBe("/api/v1/items?active=true&count=10");
  });
});

describe("Family & Account Rust-Grade Schema Deserializers", () => {
  it("deserializes valid Family, Member, and tagged Account unions", () => {
    const rawBank = {
      id: 101,
      familyId: 1,
      ownerMemberId: 1,
      type: "bank_account",
      currencyId: 1,
      bankName: "Chase",
      accountName: "Checking",
      last4: "1234",
      availableBalance: 500000,
      createdAt: 1704067200,
    };
    const bank = parseAccount(rawBank);
    expect(bank.type).toBe("bank_account");
    if (bank.type === "bank_account") {
      expect(bank.currencyId).toBe(1);
      expect(bank.bankName).toBe("Chase");
      expect(bank.accountName).toBe("Checking");
      expect(bank.availableBalance).toBe(500000);
      expect(bank.id).toBe(101);
    }

    const rawCard = {
      id: 201,
      familyId: 1,
      ownerMemberId: 1,
      type: "credit_card",
      currencyId: 2,
      bankName: "Amex",
      cardName: "Gold",
      last4: "5678",
      creditLimit: 1000000,
      availableCredit: 800000,
      outstandingBalance: 200000,
      createdAt: 1704067200,
    };
    const card = parseAccount(rawCard);
    expect(card.type).toBe("credit_card");
    if (card.type === "credit_card") {
      expect(card.currencyId).toBe(2);
      expect(card.creditLimit).toBe(1000000);
      expect(card.availableCredit).toBe(800000);
      expect(card.outstandingBalance).toBe(200000);
      expect(card.id).toBe(201);
    }
  });

  it("throws ContractViolationError on missing fields or invalid discriminator", () => {
    expect(() => parseAccount({ type: "crypto_wallet" })).toThrow(ContractViolationError);
    expect(() => parseAccount({ type: "credit_card", creditLimit: "ten thousand" })).toThrow(
      ContractViolationError,
    );
    expect(() => parseMember({ id: "invalid-string-id" })).toThrow(ContractViolationError);
  });
});
