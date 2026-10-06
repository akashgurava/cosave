import { describe, expect, it } from "vitest";
import type {
  AccountId,
  CategoryId,
  MemberId,
  MinorUnits,
  SubcategoryId,
  Transaction,
  TransactionFilters,
  TransactionId,
  TypeId,
} from "./types";
import {
  applyFilters,
  INITIAL_MOCK_TRANSACTIONS,
  parseCurrencyInput,
  resolveDatePresetToRange,
} from "./mock";

describe("mock.ts utility functions", () => {
  describe("parseCurrencyInput", () => {
    it("parses valid dollar strings into minor units integer", () => {
      expect(parseCurrencyInput("100")).toBe(10000);
      expect(parseCurrencyInput("$100.50")).toBe(10050);
      expect(parseCurrencyInput("1,250.75")).toBe(125075);
      expect(parseCurrencyInput(" 42.1 ")).toBe(4210);
    });

    it("returns null for non-numeric or invalid strings", () => {
      expect(parseCurrencyInput("")).toBeNull();
      expect(parseCurrencyInput("abc")).toBeNull();
      expect(parseCurrencyInput("---")).toBeNull();
      expect(parseCurrencyInput("0")).toBe(0);
      expect(parseCurrencyInput("-$50")).toBe(-5000);
    });
  });

  describe("applyFilters", () => {
    const baseFilters: TransactionFilters = {
      searchQuery: "",
      datePreset: "all",
      amountPreset: "all",
      selectedTypeIds: [],
      selectedMemberIds: [],
      selectedAccountIds: [],
      selectedCategoryIds: [],
      selectedSubcategoryIds: [],
      selectedStatuses: [],
    };

    const sampleTxList: readonly Transaction[] = [
      {
        id: 1 as TransactionId,
        date: "2026-10-05",
        description: "WHOLEFDS SOMA",
        payee: "Whole Foods",
        amount: -8420 as MinorUnits,
        typeId: 2 as TypeId,
        type: "Expense",
        typeColor: "#f43f5e",
        memberId: 1 as MemberId,
        accountId: 1 as AccountId,
        categoryId: 2 as CategoryId,
        subcategoryId: 201 as SubcategoryId,
        status: "cleared",
        notes: "Organic groceries",
      },
      {
        id: 2 as TransactionId,
        date: "2026-10-04",
        description: "STRIPE PAYOUT",
        payee: "Stripe",
        amount: 450000 as MinorUnits,
        typeId: 1 as TypeId,
        type: "Income",
        typeColor: "#10b981",
        memberId: 2 as MemberId,
        accountId: 3 as AccountId,
        categoryId: 1 as CategoryId,
        subcategoryId: undefined,
        status: "cleared",
      },
      {
        id: 3 as TransactionId,
        date: "2026-09-15",
        description: "INTERNAL TRANSFER",
        payee: "Transfer",
        amount: -20000 as MinorUnits,
        typeId: 3 as TypeId,
        type: "Transfer",
        typeColor: "#3b82f6",
        memberId: 1 as MemberId,
        accountId: 2 as AccountId,
        categoryId: 3 as CategoryId,
        subcategoryId: undefined,
        status: "pending",
      },
    ];

    it("returns all items when filters are empty", () => {
      const res = applyFilters(sampleTxList, baseFilters);
      expect(res.length).toBe(3);
    });

    it("filters by text search across description, payee, and notes", () => {
      const byDesc = applyFilters(sampleTxList, { ...baseFilters, searchQuery: "wholefds" });
      expect(byDesc.length).toBe(1);
      expect(byDesc[0]?.id).toBe(1);

      const byPayee = applyFilters(sampleTxList, { ...baseFilters, searchQuery: "Stripe" });
      expect(byPayee.length).toBe(1);
      expect(byPayee[0]?.id).toBe(2);

      const byNotes = applyFilters(sampleTxList, { ...baseFilters, searchQuery: "Organic" });
      expect(byNotes.length).toBe(1);
      expect(byNotes[0]?.id).toBe(1);
    });

    it("filters by type", () => {
      const incomeOnly = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedTypeIds: [1 as TypeId],
      });
      expect(incomeOnly.length).toBe(1);
      expect(incomeOnly[0]?.id).toBe(2);
    });

    it("filters by member and account", () => {
      const alexOnly = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedMemberIds: [1 as MemberId],
      });
      expect(alexOnly.length).toBe(2);

      const accOnly = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedAccountIds: [1 as AccountId],
      });
      expect(accOnly.length).toBe(1);
      expect(accOnly[0]?.id).toBe(1);
    });

    it("filters by subcategory including '(No Subcategory)' (id <= 0)", () => {
      const noSub = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedSubcategoryIds: [0 as SubcategoryId],
      });
      expect(noSub.length).toBe(2);
      expect(noSub.map((t) => t.id)).toEqual([2, 3]);

      const specificSub = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedSubcategoryIds: [201 as SubcategoryId],
      });
      expect(specificSub.length).toBe(1);
      expect(specificSub[0]?.id).toBe(1);
    });

    it("filters by status", () => {
      const pendingOnly = applyFilters(sampleTxList, {
        ...baseFilters,
        selectedStatuses: ["pending"],
      });
      expect(pendingOnly.length).toBe(1);
      expect(pendingOnly[0]?.id).toBe(3);
    });

    it("filters by custom amount min and max", () => {
      const customAmount = applyFilters(sampleTxList, {
        ...baseFilters,
        amountPreset: "custom",
        customAmountMin: 5000,
        customAmountMax: 10000,
      });
      expect(customAmount.length).toBe(1);
      expect(customAmount[0]?.id).toBe(1); // 8420 cents
    });

    it("filters by custom date range", () => {
      const dateFiltered = applyFilters(sampleTxList, {
        ...baseFilters,
        datePreset: "custom",
        customDateFrom: "2026-10-01",
        customDateTo: "2026-10-06",
      });
      expect(dateFiltered.length).toBe(2);
      expect(dateFiltered.map((t) => t.id)).toEqual([1, 2]);
    });

    it("filters by relative date presets (1d, 3d, 1m) on sample data", () => {
      // sampleTxList has: id 1 (2026-10-05), id 2 (2026-10-04), id 3 (2026-09-15)
      const oneDay = applyFilters(sampleTxList, { ...baseFilters, datePreset: "1d" }, "2026-10-05");
      expect(oneDay.length).toBe(1); // 2026-10-05 only
      expect(oneDay.map((t) => t.id)).toEqual([1]);

      const threeDays = applyFilters(
        sampleTxList,
        { ...baseFilters, datePreset: "3d" },
        "2026-10-05",
      );
      expect(threeDays.length).toBe(2); // 2026-10-03 to 2026-10-05, matches id 1 and 2
      expect(threeDays.map((t) => t.id)).toEqual([1, 2]);

      const oneMonth = applyFilters(
        sampleTxList,
        { ...baseFilters, datePreset: "1m" },
        "2026-10-05",
      );
      expect(oneMonth.length).toBe(3); // 2026-09-05 to 2026-10-05, matches all 3
    });

    it("filters directly with startDate and endDate query params", () => {
      const queryParamFiltered = applyFilters(sampleTxList, {
        ...baseFilters,
        startDate: "2026-10-01",
        endDate: "2026-10-04",
      });
      expect(queryParamFiltered.length).toBe(1);
      expect(queryParamFiltered[0]?.id).toBe(2);
    });

    it("verifies INITIAL_MOCK_TRANSACTIONS dataset contains 50 items and progressively filters by every preset", () => {
      expect(INITIAL_MOCK_TRANSACTIONS.length).toBe(50);

      const d1 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "1d" },
        "2026-10-05",
      );
      expect(d1.length).toBe(5);

      const d3 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "3d" },
        "2026-10-05",
      );
      expect(d3.length).toBe(11);

      const d7 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "7d" },
        "2026-10-05",
      );
      expect(d7.length).toBe(18);

      const m1 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "1m" },
        "2026-10-05",
      );
      expect(m1.length).toBe(26);

      const m3 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "3m" },
        "2026-10-05",
      );
      expect(m3.length).toBe(34);

      const m6 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "6m" },
        "2026-10-05",
      );
      expect(m6.length).toBe(40);

      const y1 = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "1y" },
        "2026-10-05",
      );
      expect(y1.length).toBe(46);

      const all = applyFilters(
        INITIAL_MOCK_TRANSACTIONS,
        { ...baseFilters, datePreset: "all" },
        "2026-10-05",
      );
      expect(all.length).toBe(50);
    });
  });

  describe("resolveDatePresetToRange", () => {
    it("returns empty bounds for all and custom", () => {
      expect(resolveDatePresetToRange("all")).toEqual({});
      expect(resolveDatePresetToRange("custom")).toEqual({});
    });

    it("correctly resolves 1d, 3d, 7d, 1m, 3m, 6m, 1y relative to reference date", () => {
      const ref = "2026-10-05";
      expect(resolveDatePresetToRange("1d", ref)).toEqual({
        startDate: "2026-10-05",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("3d", ref)).toEqual({
        startDate: "2026-10-03",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("7d", ref)).toEqual({
        startDate: "2026-09-29",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("1m", ref)).toEqual({
        startDate: "2026-09-05",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("3m", ref)).toEqual({
        startDate: "2026-07-05",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("6m", ref)).toEqual({
        startDate: "2026-04-05",
        endDate: "2026-10-05",
      });
      expect(resolveDatePresetToRange("1y", ref)).toEqual({
        startDate: "2025-10-05",
        endDate: "2026-10-05",
      });
    });
  });
});
