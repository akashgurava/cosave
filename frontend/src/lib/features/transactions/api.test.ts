import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, Code, ContractViolationError, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import {
  toAccountId,
  toCategoryId,
  toMinorUnits,
  toTransactionId,
  toTypeId,
  type MinorUnits,
  type TypeId,
} from "$lib/types";
import { transactionsApi } from "./api";
import type { NewTransaction, Transaction } from "./types";

const mockTransactionWireRaw = {
  id: "tx-1",
  source: "manual",
  date: "2026-10-05",
  description: "WHOLEFDS SOMA #10294",
  payee: "Whole Foods Market",
  amount: -8420,
  typeId: 2,
  accountId: 1,
  categoryId: 1,
  subcategoryId: 10,
  notes: "Groceries",
  status: "cleared",
};

describe("Transactions API Contract & Schema Enforcement", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  describe("transactionsApi.getTransactions", () => {
    it("fetches and decodes paginated transactions response", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [mockTransactionWireRaw],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));

      const res = await transactionsApi.getTransactions();
      expect(res.items).toHaveLength(1);
      expect(res.totalCount).toBe(1);
      const first = res.items[0];
      expect(first?.id).toBe("tx-1");
      expect(first?.source).toBe("manual");
      expect(first?.date).toBe("2026-10-05");
      expect(first?.description).toBe("WHOLEFDS SOMA #10294");
      expect(first?.amount).toBe(-8420);
    });

    it("serializes filter query parameters correctly", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", (req) => {
        expect(req.url).toContain("q=coffee");
        expect(req.url).toContain("startDate=2026-10-01");
        expect(req.url).toContain("page=2");
        expect(req.url).toContain("pageSize=50");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            items: [],
            totalCount: 0,
            page: 2,
            pageSize: 50,
            totalPages: 1,
          },
        };
      });

      const res = await transactionsApi.getTransactions({
        query: "coffee",
        startDate: "2026-10-01",
        page: 2,
        pageSize: 50,
      });
      expect(res.items).toEqual([]);
      expect(res.page).toBe(2);
    });

    it("throws ContractViolationError when items is not an array", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { items: "not-an-array", totalCount: 0, page: 1, pageSize: 20, totalPages: 1 },
      }));

      await expect(transactionsApi.getTransactions()).rejects.toThrow(ContractViolationError);
    });

    it("throws ContractViolationError when an item violates domain schema", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [{ ...mockTransactionWireRaw, amount: "invalid-string" }],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));

      await expect(transactionsApi.getTransactions()).rejects.toThrow(ContractViolationError);
    });

    it("propagates ApiError when server returns 500", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => {
        throw new ApiError(
          "Internal Server Error",
          500,
          500,
          "INTERNAL_ERROR",
          null,
          "TX.FETCH.FAILED",
        );
      });

      await expect(transactionsApi.getTransactions()).rejects.toThrow(ApiError);
    });
  });

  describe("transactionsApi.getTransaction", () => {
    it("fetches single transaction by id", async () => {
      memoryTransport.on("GET", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockTransactionWireRaw,
      }));

      const tx = await transactionsApi.getTransaction(toTransactionId("tx-1"));
      expect(tx.id).toBe("tx-1");
      expect(tx.description).toBe("WHOLEFDS SOMA #10294");
      expect(tx.source).toBe("manual");
    });

    it("propagates 404 ApiError when transaction is not found", async () => {
      memoryTransport.on("GET", "/api/v1/transactions/tx-999", () => {
        throw new ApiError("Not Found", 404, 404, "NOT_FOUND", null, "TX.GET.NOT_FOUND");
      });

      await expect(
        transactionsApi.getTransaction(toTransactionId("tx-999")),
      ).rejects.toThrow(ApiError);
    });
  });

  describe("transactionsApi.createTransaction", () => {
    it("sends POST request with clean payload and decodes created wire DTO", async () => {
      const payload: NewTransaction = {
        source: "manual",
        date: "2026-10-06",
        description: "Equinox Gym",
        payee: "Equinox",
        amount: toMinorUnits(-28000),
        typeId: toTypeId(2),
        accountId: toAccountId(1),
        categoryId: toCategoryId(3),
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.description).toBe("Equinox Gym");
        expect(body.amount).toBe(-28000);
        // Ensure presentation fields are NOT sent to backend
        expect(body.type).toBeUndefined();
        expect(body.typeColor).toBeUndefined();
        expect(body.memberId).toBeUndefined();
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: { ...mockTransactionWireRaw, id: "tx-created-101", description: "Equinox Gym" },
        };
      });

      const created = await transactionsApi.createTransaction(payload);
      expect(created.id).toBe("tx-created-101");
      expect(created.description).toBe("Equinox Gym");
    });

    it("propagates 400 ApiError on invalid input", async () => {
      memoryTransport.on("POST", "/api/v1/transactions", () => {
        throw new ApiError("Bad Request", 400, 400, "BAD_REQUEST", null, "TX.CREATE.VALIDATION");
      });

      await expect(
        transactionsApi.createTransaction({
          source: "manual",
          date: "2026-10-06",
          description: null,
          payee: null,
          amount: toMinorUnits(0),
          typeId: toTypeId(2),
          accountId: toAccountId(1),
          categoryId: toCategoryId(1),
          status: "cleared",
        }),
      ).rejects.toThrow(ApiError);
    });
  });

  describe("transactionsApi.updateTransaction", () => {
    it("sends PATCH request with source and updates, and decodes response", async () => {
      const tx: Transaction = {
        id: toTransactionId("tx-1"),
        source: "manual",
        date: "2026-10-05",
        description: "WHOLEFDS SOMA #10294",
        payee: "Whole Foods Organic Market",
        amount: toMinorUnits(-9250),
        typeId: toTypeId(2),
        accountId: toAccountId(1),
        categoryId: toCategoryId(1),
        status: "cleared",
      };

      memoryTransport.on("PATCH", "/api/v1/transactions/tx-1", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.id).toBeUndefined();
        expect(body.source).toBe("manual");
        expect(body.payee).toBe("Whole Foods Organic Market");
        expect(body.amount).toBe(-9250);
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            ...mockTransactionWireRaw,
            payee: "Whole Foods Organic Market",
            amount: -9250,
          },
        };
      });

      const updated = await transactionsApi.updateTransaction(tx);
      expect(updated.id).toBe("tx-1");
      expect(updated.payee).toBe("Whole Foods Organic Market");
      expect(updated.amount).toBe(-9250);
    });
  });

  describe("transactionsApi.deleteTransaction", () => {
    it("sends DELETE request with body containing source and decodes null response", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/tx-1", (req) => {
        expect(req.body).toBe(JSON.stringify({ source: "manual" }));
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const res = await transactionsApi.deleteTransaction(toTransactionId("tx-1"), "manual");
      expect(res).toBeNull();
    });

    it("propagates 404 ApiError when deleting non-existent transaction", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/tx-999", () => {
        throw new ApiError("Not Found", 404, 404, "NOT_FOUND", null, "TX.DELETE.NOT_FOUND");
      });

      await expect(
        transactionsApi.deleteTransaction(toTransactionId("tx-999"), "manual"),
      ).rejects.toThrow(ApiError);
    });
  });
});
