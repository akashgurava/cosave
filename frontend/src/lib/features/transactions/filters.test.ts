import { describe, expect, it } from "vitest";
import {
  applyFilters,
  getDatePresetCutoff,
  parseCurrencyInput,
  resolveDatePresetToRange,
} from "./filters";
import type {
  AccountId,
  CategoryId,
  MemberId,
  MinorUnits,
  Transaction,
  TransactionFilters,
  TransactionId,
  TypeId,
} from "./types";

const sampleTransactions: readonly Transaction[] = Object.freeze([
  {
    id: "tx-1" as TransactionId,
    source: "manual",
    date: "2026-10-05",
    description: "WHOLE FOODS SOMA",
    payee: "Whole Foods Market",
    amount: -8420 as MinorUnits,
    typeId: 2 as TypeId,
    type: "Expense",
    typeColor: "#f43f5e",
    memberId: 1 as MemberId,
    accountId: 1 as AccountId,
    categoryId: 3 as CategoryId,
    status: "cleared",
  },
  {
    id: "tx-2" as TransactionId,
    source: "manual",
    date: "2026-10-01",
    description: "PAYCHECK SALARY",
    payee: "Employer Inc",
    amount: 350000 as MinorUnits,
    typeId: 1 as TypeId,
    type: "Income",
    typeColor: "#10b981",
    memberId: 2 as MemberId,
    accountId: 2 as AccountId,
    categoryId: 1 as CategoryId,
    status: "cleared",
  },
]);

const defaultFilters: TransactionFilters = Object.freeze({
  searchQuery: "",
  datePreset: "all",
  amountPreset: "all",
  selectedMemberIds: [],
  selectedAccountIds: [],
  selectedTypeIds: [],
  selectedCategoryIds: [],
  selectedSubcategoryIds: [],
  selectedStatuses: [],
});

describe("Transaction Filters & Utilities (filters.ts)", () => {
  describe("resolveDatePresetToRange & getDatePresetCutoff", () => {
    it("resolves date presets relative to anchor date", () => {
      const range7d = resolveDatePresetToRange("7d", "2026-10-05");
      expect(range7d.endDate).toBe("2026-10-05");
      expect(range7d.startDate).toBe("2026-09-29");

      const rangeAll = resolveDatePresetToRange("all");
      expect(rangeAll.startDate).toBeUndefined();
      expect(rangeAll.endDate).toBeUndefined();
    });

    it("returns cutoff date string for preset", () => {
      expect(getDatePresetCutoff("7d", "2026-10-05")).toBe("2026-09-29");
      expect(getDatePresetCutoff("all")).toBeNull();
    });
  });

  describe("parseCurrencyInput", () => {
    it("parses currency strings into minor units integer", () => {
      expect(parseCurrencyInput("$12.34")).toBe(1234);
      expect(parseCurrencyInput("100")).toBe(10000);
      expect(parseCurrencyInput("-45.50")).toBe(-4550);
      expect(parseCurrencyInput("")).toBeNull();
      expect(parseCurrencyInput("invalid")).toBeNull();
    });
  });

  describe("applyFilters", () => {
    it("filters transactions by text search query", () => {
      const filtered = applyFilters(sampleTransactions, {
        ...defaultFilters,
        searchQuery: "Whole Foods",
      });
      expect(filtered).toHaveLength(1);
      expect(filtered[0]?.id).toBe("tx-1");
    });

    it("filters transactions by memberId", () => {
      const filtered = applyFilters(sampleTransactions, {
        ...defaultFilters,
        selectedMemberIds: [2 as MemberId],
      });
      expect(filtered).toHaveLength(1);
      expect(filtered[0]?.id).toBe("tx-2");
    });

    it("filters transactions by amount preset", () => {
      const filtered = applyFilters(sampleTransactions, {
        ...defaultFilters,
        amountPreset: "lt100",
      });
      expect(filtered).toHaveLength(1);
      expect(filtered[0]?.id).toBe("tx-1");
    });

    it("filters transactions with null description gracefully", () => {
      const txWithNullDesc: Transaction = {
        ...sampleTransactions[0]!,
        id: "tx-null-desc" as TransactionId,
        description: null,
        payee: "Coffee Shop",
      };
      const filtered = applyFilters([txWithNullDesc], {
        ...defaultFilters,
        searchQuery: "Coffee",
      });
      expect(filtered).toHaveLength(1);

      const filteredNone = applyFilters([txWithNullDesc], {
        ...defaultFilters,
        searchQuery: "Nonexistent",
      });
      expect(filteredNone).toHaveLength(0);
    });
  });
});
