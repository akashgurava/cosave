import { describe, it, expect, beforeEach } from "vitest";
import { api, ApiError, Code, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { TransactionsStore } from "./store.svelte";
import { transactionsApi } from "./api";
import type {
  CreateTransactionInput,
  MemberId,
  MinorUnits,
  TransactionId,
  TransactionWireDto,
  TypeId,
} from "./types";

const sampleWireTx1: TransactionWireDto = {
  id: "tx-1",
  source: "manual",
  date: "2026-10-05",
  description: "Whole Foods Market",
  payee: "Whole Foods",
  amount: -8420,
  typeId: 2,
  accountId: 1,
  categoryId: 1,
  subcategoryId: 10,
  notes: null,
  status: "cleared",
};

const sampleWireTx2: TransactionWireDto = {
  id: "tx-2",
  source: "manual",
  date: "2026-10-05",
  description: "Acme Corp Payroll",
  payee: "Acme Corp",
  amount: 485000,
  typeId: 1,
  accountId: 2,
  categoryId: 2,
  subcategoryId: null,
  notes: null,
  status: "cleared",
};

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
          items: [sampleWireTx1, sampleWireTx2],
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
          items: [sampleWireTx1],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("creates a transaction and prepends to the collection", async () => {
      const input: CreateTransactionInput = {
        date: "2026-10-06",
        description: "Target Groceries",
        payee: "Target",
        amount: -5420 as MinorUnits,
        typeId: 2 as TypeId,
        accountId: 1,
        categoryId: 1,
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          ...sampleWireTx1,
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

    it("updates a transaction in-place and clears draft", async () => {
      memoryTransport.on("PATCH", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { ...sampleWireTx1, payee: "Whole Foods Organic" },
      }));

      store.setRowDraftField("tx-1" as TransactionId, "payee", "Whole Foods Organic");
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(true);

      const updated = await store.update("tx-1" as TransactionId, {
        payee: "Whole Foods Organic",
      });
      expect(updated.payee).toBe("Whole Foods Organic");
      expect(store.transactions[0]?.payee).toBe("Whole Foods Organic");
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);
    });

    it("deletes a transaction from the store and clears drafts", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: null,
      }));

      await store.delete("tx-1" as TransactionId);
      expect(store.transactions).toHaveLength(0);
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);
    });
  });

  describe("Filtering & Sorting", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          items: [sampleWireTx1, sampleWireTx2],
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
      store.setFilter("selectedTypeIds", [2 as TypeId]);
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
          items: [sampleWireTx1, sampleWireTx2],
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
          items: [sampleWireTx1],
          totalCount: 1,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        },
      }));
      await store.load();
    });

    it("stages and detects row draft differences", () => {
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);

      // Staging identical value does NOT mark as dirty
      store.setRowDraftField("tx-1" as TransactionId, "payee", "Whole Foods");
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);

      // Staging new value marks as dirty
      store.setRowDraftField("tx-1" as TransactionId, "payee", "Trader Joe's");
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(true);

      const original = store.getTransaction("tx-1" as TransactionId);
      const effective = store.getEffectiveTx(original);
      expect(effective.payee).toBe("Trader Joe's");
    });

    it("discards row drafts cleanly", () => {
      store.setRowDraftField("tx-1" as TransactionId, "payee", "Trader Joe's");
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(true);

      store.discardRowDraft("tx-1" as TransactionId);
      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);
      const original = store.getTransaction("tx-1" as TransactionId);
      expect(store.getEffectiveTx(original).payee).toBe("Whole Foods");
    });

    it("saves staged row draft via transport update", async () => {
      memoryTransport.on("PATCH", "/api/v1/transactions/tx-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { ...sampleWireTx1, payee: "Trader Joe's" },
      }));

      store.setRowDraftField("tx-1" as TransactionId, "payee", "Trader Joe's");
      await store.saveRowDraft("tx-1" as TransactionId);

      expect(store.hasRowDraft("tx-1" as TransactionId)).toBe(false);
      expect(store.transactions[0]?.payee).toBe("Trader Joe's");
    });
  });
});
