import { describe, expect, it } from "vitest";
import {
  applyFilters,
  getDatePresetCutoff,
  getTodayIsoDate,
  parseCurrencyInput,
  resolveDatePresetToRange,
} from "./filters";
import type { AccountId, MemberId, Transaction, TransactionFilters, TransactionId } from "./types";
import { toAccountId, toCategoryId, toMinorUnits, toTransactionId, toTypeId } from "$lib/types";

const sampleTransactions: readonly Transaction[] = Object.freeze([
  {
    id: toTransactionId("tx-1"),
    source: "manual",
    date: "2026-10-05",
    description: "WHOLE FOODS SOMA",
    payee: "Whole Foods Market",
    amount: toMinorUnits(-8420),
    typeId: toTypeId(2),
    accountId: toAccountId(1),
    categoryId: toCategoryId(3),
    status: "cleared",
  },
  {
    id: toTransactionId("tx-2"),
    source: "manual",
    date: "2026-10-01",
    description: "PAYCHECK SALARY",
    payee: "Employer Inc",
    amount: toMinorUnits(350000),
    typeId: toTypeId(1),
    accountId: toAccountId(2),
    categoryId: toCategoryId(1),
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
    it("resolves date presets relative to anchor date and defaults to today", () => {
      const today = getTodayIsoDate();
      expect(today).toMatch(/^\d{4}-\d{2}-\d{2}$/);

      const defaultRange = resolveDatePresetToRange("1d");
      expect(defaultRange.endDate).toBe(today);
      expect(defaultRange.startDate).toBe(today);

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
      const getMemberId = (accId: AccountId) =>
        accId === (1 as AccountId) ? (1 as MemberId) : (2 as MemberId);
      const filtered = applyFilters(
        sampleTransactions,
        {
          ...defaultFilters,
          selectedMemberIds: [2 as MemberId],
        },
        getMemberId,
      );
      expect(filtered).toHaveLength(1);
      expect(filtered[0]?.id).toBe("tx-2");
    });

    it("filters transactions by custom amount range", () => {
      const filtered = applyFilters(sampleTransactions, {
        ...defaultFilters,
        amountPreset: "custom",
        customAmountMax: 100 * 100,
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
