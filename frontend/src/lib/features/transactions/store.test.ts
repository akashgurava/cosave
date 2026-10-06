import { describe, it, expect, beforeEach } from "vitest";
import { api, ApiError, Code, MemoryTransportAdapter, Status } from "$lib/api";
import { TransactionsStore } from "./store.svelte";
import { transactionsApi } from "./api";
import type {
  AccountId,
  CategoryId,
  CreateTransactionInput,
  MemberId,
  MinorUnits,
  SubcategoryId,
  Transaction,
  TransactionId,
  TypeId,
} from "./types";

const sampleTx: Transaction = {
  id: 1 as TransactionId,
  date: "2026-10-05",
  description: "Whole Foods Market",
  payee: "Whole Foods",
  amount: -8420 as MinorUnits,
  typeId: 2 as TypeId,
  type: "Expense",
  typeColor: "#f43f5e",
  memberId: 1 as MemberId,
  accountId: 1 as AccountId,
  categoryId: 1 as CategoryId,
  subcategoryId: 10 as SubcategoryId,
  status: "cleared",
};

const sampleIncomeTx: Transaction = {
  id: 2 as TransactionId,
  date: "2026-10-05",
  description: "Acme Corp Payroll",
  payee: "Acme Corp",
  amount: 485000 as MinorUnits,
  typeId: 1 as TypeId,
  type: "Income",
  typeColor: "#10b981",
  memberId: 1 as MemberId,
  accountId: 2 as AccountId,
  categoryId: 2 as CategoryId,
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
    it("starts with loaded state from initial mock data", () => {
      expect(store.isLoaded).toBe(true);
      expect(store.transactions.length).toBeGreaterThan(0);
    });

    it("loads transactions via transport and updates state to success", async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [sampleTx, sampleIncomeTx],
      }));

      await store.load();
      expect(store.isLoaded).toBe(true);
      expect(store.transactions).toHaveLength(2);
      expect(store.transactions[0]?.description).toBe("Whole Foods Market");
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
        data: [sampleTx],
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
        type: "Expense",
        typeColor: "#f43f5e",
        memberId: 1,
        accountId: 1,
        categoryId: 1,
        status: "cleared",
      };

      memoryTransport.on("POST", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { ...input, id: 99 },
      }));

      const created = await store.create(input);
      expect(created.id).toBe(99);
      expect(store.transactions).toHaveLength(2);
      expect(store.transactions[0]?.id).toBe(99);
    });

    it("updates a transaction in-place and clears draft", async () => {
      memoryTransport.on("PATCH", "/api/v1/transactions/1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { ...sampleTx, payee: "Whole Foods Organic" },
      }));

      store.setRowDraftField(1 as TransactionId, "payee", "Whole Foods Organic");
      expect(store.hasRowDraft(1 as TransactionId)).toBe(true);

      const updated = await store.update(1, { payee: "Whole Foods Organic" });
      expect(updated.payee).toBe("Whole Foods Organic");
      expect(store.transactions[0]?.payee).toBe("Whole Foods Organic");
      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);
    });

    it("deletes a transaction from the store and clears drafts", async () => {
      memoryTransport.on("DELETE", "/api/v1/transactions/1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: null,
      }));

      await store.delete(1);
      expect(store.transactions).toHaveLength(0);
      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);
    });
  });

  describe("Filtering & Sorting", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [sampleTx, sampleIncomeTx],
      }));
      await store.load();
    });

    it("filters transactions by search query", () => {
      store.setFilter("searchQuery", "payroll");
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe(2);
    });

    it("filters transactions by type", () => {
      store.setFilter("selectedTypeIds", [2 as TypeId]);
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe(1);
    });

    it("filters transactions by memberId", () => {
      store.setFilter("selectedMemberIds", [99 as MemberId]);
      expect(store.filteredTransactions).toHaveLength(0);
    });

    it("filters transactions by amount range slider", () => {
      // 0 to 1 means $0 to $100
      store.amountPointRange = [0, 1];
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe(1); // $84.20 is < $100

      // 4 to 5 means $2000 to > $2000
      store.amountPointRange = [4, 5];
      expect(store.filteredTransactions).toHaveLength(1);
      expect(store.filteredTransactions[0]?.id).toBe(2); // $4,850 is > $2000
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
      expect(store.filteredTransactions[0]?.id).toBe(2); // 485000 > -8420

      store.setSort("amount"); // toggle to asc
      expect(store.filteredTransactions[0]?.id).toBe(1); // -8420 < 485000
    });
  });

  describe("Timeline Grouping & Financial Net Sums", () => {
    beforeEach(async () => {
      memoryTransport.on("GET", "/api/v1/transactions", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [sampleTx, sampleIncomeTx],
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
        data: [sampleTx],
      }));
      await store.load();
    });

    it("stages and detects row draft differences", () => {
      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);

      // Staging identical value does NOT mark as dirty
      store.setRowDraftField(1 as TransactionId, "payee", "Whole Foods");
      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);

      // Staging new value marks as dirty
      store.setRowDraftField(1 as TransactionId, "payee", "Trader Joe's");
      expect(store.hasRowDraft(1 as TransactionId)).toBe(true);

      const effective = store.getEffectiveTx(sampleTx);
      expect(effective.payee).toBe("Trader Joe's");
    });

    it("discards row drafts cleanly", () => {
      store.setRowDraftField(1 as TransactionId, "payee", "Trader Joe's");
      expect(store.hasRowDraft(1 as TransactionId)).toBe(true);

      store.discardRowDraft(1 as TransactionId);
      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);
      expect(store.getEffectiveTx(sampleTx).payee).toBe("Whole Foods");
    });

    it("saves staged row draft via transport update", async () => {
      memoryTransport.on("PATCH", "/api/v1/transactions/1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { ...sampleTx, payee: "Trader Joe's" },
      }));

      store.setRowDraftField(1 as TransactionId, "payee", "Trader Joe's");
      await store.saveRowDraft(1 as TransactionId);

      expect(store.hasRowDraft(1 as TransactionId)).toBe(false);
      expect(store.transactions[0]?.payee).toBe("Trader Joe's");
    });
  });
});
