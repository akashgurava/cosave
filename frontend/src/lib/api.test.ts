import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  api,
  apiFetch,
  ApiError,
  ContractViolationError,
  MemoryTransportAdapter,
  Code,
  Status,
  buildUrl,
} from "./api";
import { parseAccount, parseMember } from "./features/family/types";

describe("Deepened ApiClient (Caller-Optimized REST Client)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  it("unwraps successful backend envelope and directly returns typed data", async () => {
    memoryTransport.on("GET", "/api/v1/auth/me", () => ({
      code: 0,
      status: "OK",
      data: { id: "usr-1", name: "Alice", role: "admin", created_at: 1700000000 },
    }));

    const user = await api.get<{ id: string; name: string }>("/api/v1/auth/me");
    expect(user.id).toBe("usr-1");
    expect(user.name).toBe("Alice");
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
    expect(txs[0].id).toBe("tx-1");
  });

  it("throws normalized ApiError on 401 unauthenticated with isUnauthorized", async () => {
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
    expect(error?.httpStatus).toBe(401);
    expect(error?.code).toBe(401);
    expect(error?.apiStatus).toBe("UNAUTHENTICATED");
    expect(error?.isUnauthorized).toBe(true);
  });

  it("throws normalized ApiError on 409 conflict with isConflict", async () => {
    memoryTransport.on("POST", "/api/v1/auth/register", () => ({
      code: 409,
      status: "USER_EXISTS",
      data: null,
    }));

    let error: ApiError | null = null;
    try {
      await api.post("/api/v1/auth/register", { name: "alice", password: "pwd" });
    } catch (err) {
      error = err as ApiError;
    }

    expect(error).toBeInstanceOf(ApiError);
    expect(error?.code).toBe(409);
    expect(error?.apiStatus).toBe("USER_EXISTS");
    expect(error?.isConflict).toBe(true);
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
    expect(error?.httpStatus).toBe(409);
    expect(error?.code).toBe(409);
    expect(error?.apiStatus).toBe("TYPE_ALREADY_EXISTS");
    expect(error?.message).toBe("Transaction type 'Income' already exists.");
    expect(error?.action).toBe("CONFIG.CATEGORIES.CREATE_TYPE");
    expect(error?.isConflict).toBe(true);
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

  it("supports apiFetch backwards compatibility wrapper", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 0,
      status: "HEALTHY",
      data: { service: "cosave" },
    }));

    const res = await apiFetch<{ service: string }>("/api/v1/health");
    expect(res.code).toBe(Code.Zero);
    expect(res.status).toBe(Status.Healthy);
    expect(res.data.service).toBe("cosave");
  });

  it("preserves OK status in apiFetch envelope when backend returns OK", async () => {
    memoryTransport.on("GET", "/api/v1/ping", () => ({
      code: 0,
      status: "OK",
      data: { pong: true },
    }));

    const res = await apiFetch<{ pong: boolean }>("/api/v1/ping");
    expect(res.code).toBe(Code.Zero);
    expect(res.status).toBe(Status.Ok);
    expect(res.data.pong).toBe(true);
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
      id: "acc-1",
      familyId: "fam-1",
      ownerMemberId: "mem-1",
      type: "bank_account",
      bankName: "Chase",
      last4: "1234",
      createdAt: "2026-01-01",
    };
    const bank = parseAccount(rawBank);
    expect(bank.type).toBe("bank_account");
    if (bank.type === "bank_account") {
      expect(bank.bankName).toBe("Chase");
    }

    const rawCard = {
      id: "acc-2",
      familyId: "fam-1",
      ownerMemberId: "mem-1",
      type: "credit_card",
      bankName: "Amex",
      cardName: "Gold",
      last4: "5678",
      creditLimit: 10000,
      createdAt: "2026-01-01",
    };
    const card = parseAccount(rawCard);
    expect(card.type).toBe("credit_card");
    if (card.type === "credit_card") {
      expect(card.creditLimit).toBe(10000);
    }
  });

  it("throws ContractViolationError on missing fields or invalid discriminator", () => {
    expect(() => parseAccount({ type: "crypto_wallet" })).toThrow(ContractViolationError);
    expect(() => parseAccount({ type: "credit_card", creditLimit: "ten thousand" })).toThrow(
      ContractViolationError,
    );
    expect(() => parseMember({ id: 123 })).toThrow(ContractViolationError);
  });
});
