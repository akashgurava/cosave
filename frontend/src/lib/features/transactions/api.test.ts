import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, Code, ContractViolationError, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/testing";
import type { MinorUnits, TypeId } from "$lib/types/core";
import { transactionsApi } from "./api";
import type { CreateTransactionInput, UpdateTransactionInput } from "./types";

const mockTransactionRaw = {
  id: 1,
  date: "2026-10-05",
  description: "WHOLEFDS SOMA #10294",
  payee: "Whole Foods Market",
  amount: -8420,
  typeId: 2,
  type: "Expense",
  typeColor: "#f43f5e",
  memberId: 1,
  accountId: 1,
  categoryId: 1,
  subcategoryId: 10,
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
    it("fetches and decodes transactions array correctly", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [mockTransactionRaw],
      }));

      const txs = await transactionsApi.getTransactions();
      expect(txs).toHaveLength(1);
      const first = txs[0];
      expect(first?.id).toBe(1);
      expect(first?.date).toBe("2026-10-05");
      expect(first?.description).toBe("WHOLEFDS SOMA #10294");
      expect(first?.amount).toBe(-8420);
      expect(first?.type).toBe("Expense");
    });

    it("serializes filter query parameters correctly", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", (req) => {
        expect(req.url).toContain("query=coffee");
        expect(req.url).toContain("from_date=2026-10-01");
        expect(req.url).toContain("type=expense");
        expect(req.url).toContain("member_id=2");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: [],
        };
      });

      const txs = await transactionsApi.getTransactions({
        query: "coffee",
        fromDate: "2026-10-01",
        type: "expense",
        memberId: 2,
      });
      expect(txs).toEqual([]);
    });

    it("throws ContractViolationError when response is not an array", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { items: "not-an-array" },
      }));

      await expect(transactionsApi.getTransactions()).rejects.toThrow(ContractViolationError);
    });

    it("throws ContractViolationError when an item violates domain schema", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [{ ...mockTransactionRaw, amount: "invalid-string" }],
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
      memoryTransport.on("GET", "/api/v1/transactions/1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockTransactionRaw,
      }));

      const tx = await transactionsApi.getTransaction(1);
      expect(tx.id).toBe(1);
      expect(tx.description).toBe("WHOLEFDS SOMA #10294");
    });

    it("propagates 404 ApiError when transaction is not found", async () => {
      memoryTransport.on("GET", "/api/v1/transactions/999", () => {
        throw new ApiError("Not Found", 404, 404, "NOT_FOUND", null, "TX.GET.NOT_FOUND");
      });

      await expect(transactionsApi.getTransaction(999)).rejects.toThrow(ApiError);
    });
  });

  describe("transactionsApi.createTransaction", () => {
    it("sends POST request and decodes created transaction", async () => {
      const payload: CreateTransactionInput = {
        date: "2026-10-06",
        description: "Equinox Gym",
        payee: "Equinox",
        amount: -28000 as MinorUnits,
        typeId: 2 as TypeId,
        type: "Expense",
        typeColor: "#f43f5e",
        memberId: 1,
        accountId: 1,
        categoryId: 3,
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.description).toBe("Equinox Gym");
        expect(body.amount).toBe(-28000);
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: { ...payload, id: 101 },
        };
      });

      const created = await transactionsApi.createTransaction(payload);
      expect(created.id).toBe(101);
      expect(created.description).toBe("Equinox Gym");
      expect(created.amount).toBe(-28000);
    });

    it("propagates 400 ApiError on invalid input", async () => {
      memoryTransport.on("POST", "/api/v1/transactions", () => {
        throw new ApiError("Bad Request", 400, 400, "BAD_REQUEST", null, "TX.CREATE.VALIDATION");
      });

      await expect(
        transactionsApi.createTransaction({
          date: "2026-10-06",
          amount: 0 as MinorUnits,
          typeId: 2 as TypeId,
          type: "Expense",
          memberId: 1,
          accountId: 1,
          categoryId: 1,
        }),
      ).rejects.toThrow(ApiError);
    });
  });

  describe("transactionsApi.updateTransaction", () => {
    it("sends PATCH request with update payload and decodes response", async () => {
      const updates: UpdateTransactionInput = {
        payee: "Whole Foods Organic Market",
        amount: -9250 as MinorUnits,
      };

      memoryTransport.on("PATCH", "/api/v1/transactions/1", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.payee).toBe("Whole Foods Organic Market");
        expect(body.amount).toBe(-9250);
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            ...mockTransactionRaw,
            payee: "Whole Foods Organic Market",
            amount: -9250,
          },
        };
      });

      const updated = await transactionsApi.updateTransaction(1, updates);
      expect(updated.id).toBe(1);
      expect(updated.payee).toBe("Whole Foods Organic Market");
      expect(updated.amount).toBe(-9250);
    });
  });

  describe("transactionsApi.deleteTransaction", () => {
    it("sends DELETE request and decodes null response", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: null,
      }));

      const res = await transactionsApi.deleteTransaction(1);
      expect(res).toBeNull();
    });

    it("propagates 404 ApiError when deleting non-existent transaction", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/999", () => {
        throw new ApiError("Not Found", 404, 404, "NOT_FOUND", null, "TX.DELETE.NOT_FOUND");
      });

      await expect(transactionsApi.deleteTransaction(999)).rejects.toThrow(ApiError);
    });
  });
});
