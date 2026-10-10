import { describe, it, expect, beforeEach } from "vitest";
import { api, ApiError, Code, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { TransactionsStore } from "./store.svelte";
import { transactionsApi } from "./api";
import {
  toAccountId,
  toCategoryId,
  toMinorUnits,
  toTransactionId,
  toTypeId,
  type NewTransaction,
  type Transaction,
  type TransactionId,
  type TypeId,
} from "./types";

const sampleTx1: Transaction = Object.freeze({
  id: toTransactionId("tx-1"),
  source: "manual",
  date: "2026-10-05",
  description: "Whole Foods Market",
  payee: "Whole Foods",
  amount: toMinorUnits(-8420),
  typeId: toTypeId(2),
  accountId: toAccountId(1),
  categoryId: toCategoryId(1),
  subcategoryId: undefined,
  notes: undefined,
  status: "cleared",
});

const sampleTx2: Transaction = Object.freeze({
  id: toTransactionId("tx-2"),
  source: "manual",
  date: "2026-10-05",
  description: "Acme Corp Payroll",
  payee: "Acme Corp",
  amount: toMinorUnits(485000),
  typeId: toTypeId(1),
  accountId: toAccountId(2),
  categoryId: toCategoryId(2),
  subcategoryId: undefined,
  notes: undefined,
  status: "cleared",
});

describe("TransactionsStore (Svelte 5 Rune Domain Store)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let store: TransactionsStore;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    api.setTransport(memoryTransport);
    store = new TransactionsStore(transactionsApi);
  });

  describe("Initial State & Transport Loading", () => {
    it("starts with idle state and empty transaction array", () => {
      expect(store.state.status).toBe("idle");
      expect(store.isLoaded).toBe(false);
      expect(store.isLoading).toBe(false);
      expect(store.transactions).toEqual([]);
    });

    it("loads transactions via transport and updates state to success", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleTx1, sampleTx2],
          totalCount: 2,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));

      await store.load();
      expect(store.isLoaded).toBe(true);
      expect(store.transactions).toHaveLength(2);
      expect(store.transactions[0]?.id).toBe("tx-1");
      expect(store.transactions[0]?.description).toBe("Whole Foods Market");
      expect(store.totalCount).toBe(2);
    });

    it("transitions to error state when transport fails", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => {
        throw new ApiError("Database error", 500, 500, "INTERNAL_ERROR", null, "TX.FETCH.ERROR");
      });

      await store.load();
      expect(store.state.status).toBe("error");
      expect(store.error).toBe("Database error");
    });
  });

  describe("CRUD Operations", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleTx1],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("creates a transaction and prepends to the collection", async () => {
      const input: NewTransaction = {
        source: "manual",
        date: "2026-10-06",
        description: "Target Groceries",
        payee: "Target",
        amount: toMinorUnits(-5420),
        typeId: toTypeId(2),
        accountId: toAccountId(1),
        categoryId: toCategoryId(1),
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          ...sampleTx1,
          id: "tx-created-99",
          description: "Target Groceries",
          amount: -5420,
        },
      }));

      const created = await store.create(input);
      expect(created.id).toBe("tx-created-99");
      expect(store.transactions).toHaveLength(2);
      expect(store.transactions[0]?.id).toBe("tx-created-99");
    });

    it("creates a transaction with null description", async () => {
      const input: NewTransaction = {
        source: "manual",
        date: "2026-10-06",
        description: null,
        payee: "Target",
        amount: toMinorUnits(-5420),
        typeId: toTypeId(2),
        accountId: toAccountId(1),
        categoryId: toCategoryId(1),
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          ...sampleTx1,
          id: "tx-created-100",
          description: null,
          amount: -5420,
        },
      }));

      const created = await store.create(input);
      expect(created.id).toBe("tx-created-100");
      expect(created.description).toBeNull();
      expect(store.transactions[0]?.description).toBeNull();
    });

    it("updates a transaction in-place and clears draft", async () => {
      const updatedTx: Transaction = {
        ...sampleTx1,
        payee: "Whole Foods Organic",
      };

      memoryTransport.on("PATCH", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updatedTx,
      }));

      store.setRowDraft(updatedTx);
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(true);

      const res = await store.update(updatedTx);
      expect(res.payee).toBe("Whole Foods Organic");
      expect(store.transactions[0]?.payee).toBe("Whole Foods Organic");
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);
    });

    it("deletes a transaction from the store and clears drafts", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: null,
      }));

      await store.delete(toTransactionId("tx-1"));
      expect(store.transactions).toHaveLength(0);
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);
    });
  });

  describe("Filtering & Sorting", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleTx1, sampleTx2],
          totalCount: 2,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("filters transactions by search query", () => {
      store.setFilter("searchQuery", "payroll");
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe("tx-2");
    });

    it("filters transactions by type", () => {
      store.setFilter("selectedTypeIds", [toTypeId(2)]);
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe("tx-1");
    });

    it("filters transactions by amount range slider", () => {
      // 0 to 1 means $0 to $100
      store.amountPointRange = [0, 1];
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe("tx-1"); // $84.20 is < $100

      // 4 to 5 means $2000 to > $2000
      store.amountPointRange = [4, 5];
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe("tx-2"); // $4,850 is > $2000
    });

    it("resets all filters back to defaults", () => {
      store.setFilter("searchQuery", "test");
      store.amountPointRange = [1, 3];
      expect(store.hasActiveFilters).toBe(true);

      store.resetFilters();
      expect(store.hasActiveFilters).toBe(false);
      expect(store.filteredTransactions).toHaveLength(2);
    });

    it("sorts transactions by amount descending and ascending", () => {
      store.setSort("amount"); // first click on new field defaults to desc
      expect(store.filteredTransactions[0]?.id).toBe("tx-2"); // 485000 > -8420

      store.setSort("amount"); // toggle to asc
      expect(store.filteredTransactions[0]?.id).toBe("tx-1"); // -8420 < 485000
    });
  });

  describe("Timeline Grouping & Financial Net Sums", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleTx1, sampleTx2],
          totalCount: 2,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("computes financial net sum correctly (income positive, expense negative)", () => {
      const groups = store.timelineGroups;
      expect(groups).toHaveLength(1);
      const todayGroup = groups[0];
      expect(todayGroup?.title).toBe("Today");
      expect(todayGroup?.items).toHaveLength(2);
      // Net: +485000 - 8420 = +476580
      expect(todayGroup?.net).toBe(485000 - 8420);
    });

    it("toggles collapse state of timeline groups", () => {
      expect(store.collapsedGroupKeys.has("today")).toBe(false);
      store.toggleGroupCollapse("today");
      expect(store.collapsedGroupKeys.has("today")).toBe(true);
      store.toggleGroupCollapse("today");
      expect(store.collapsedGroupKeys.has("today")).toBe(false);
    });
  });

  describe("In-Place Row Draft Staging", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleTx1],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("stages and detects row draft differences", () => {
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);

      // Staging identical value does NOT mark as dirty
      store.setRowDraft(sampleTx1);
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);

      // Staging new value marks as dirty
      const modified: Transaction = {
        ...sampleTx1,
        payee: "Trader Joe's",
      };
      store.setRowDraft(modified);
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(true);

      const original = store.getTransaction(toTransactionId("tx-1"));
      const effective = store.getEffectiveTx(original);
      expect(effective.payee).toBe("Trader Joe's");
    });

    it("discards row drafts cleanly", () => {
      const modified: Transaction = {
        ...sampleTx1,
        payee: "Trader Joe's",
      };
      store.setRowDraft(modified);
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(true);

      store.discardRowDraft(toTransactionId("tx-1"));
      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);
      const original = store.getTransaction(toTransactionId("tx-1"));
      expect(store.getEffectiveTx(original).payee).toBe("Whole Foods");
    });

    it("saves staged row draft via transport update", async () => {
      const updatedTx: Transaction = {
        ...sampleTx1,
        payee: "Trader Joe's",
      };

      memoryTransport.on("PATCH", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updatedTx,
      }));

      store.setRowDraft(updatedTx);
      await store.saveRowDraft(toTransactionId("tx-1"));

      expect(store.hasRowDraft(toTransactionId("tx-1"))).toBe(false);
      expect(store.transactions[0]?.payee).toBe("Trader Joe's");
    });
  });
});
