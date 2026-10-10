import { describe, expect, it } from "vitest";
import { ContractViolationError } from "$lib/api";
import {
  parsePaginatedTransactions,
  parseTransaction,
  type PaginatedTransactionsDto,
  type Transaction,
} from "./types";

describe("Transactions Domain Decoders (types.ts)", () => {
  describe("parseTransaction", () => {
    it("decodes valid raw JSON into frozen Transaction entity", () => {
      const raw = {
        id: "tx-manual-101",
        source: "manual",
        date: "2026-10-05",
        description: "WHOLEFDS SOMA #10294",
        payee: "Whole Foods Market",
        amount: -8420,
        typeId: 2,
        accountId: 1,
        categoryId: 3,
        subcategoryId: 10,
        notes: "Organic groceries",
        status: "cleared",
      };

      const tx: Transaction = parseTransaction(raw);
      expect(tx.id).toBe("tx-manual-101");
      expect(tx.source).toBe("manual");
      expect(tx.date).toBe("2026-10-05");
      expect(tx.description).toBe("WHOLEFDS SOMA #10294");
      expect(tx.payee).toBe("Whole Foods Market");
      expect(tx.amount).toBe(-8420);
      expect(tx.typeId).toBe(2);
      expect(tx.accountId).toBe(1);
      expect(tx.categoryId).toBe(3);
      expect(tx.subcategoryId).toBe(10);
      expect(tx.notes).toBe("Organic groceries");
      expect(tx.status).toBe("cleared");
      expect(Object.isFrozen(tx)).toBe(true);
    });

    it("handles null payee, null subcategoryId, and null notes", () => {
      const raw = {
        id: "tx-imported-202",
        source: "import",
        date: "2026-10-06",
        description: "ATM WITHDRAWAL",
        payee: null,
        amount: -10000,
        typeId: 2,
        accountId: 1,
        categoryId: 3,
        subcategoryId: null,
        notes: null,
        status: "pending",
      };

      const tx = parseTransaction(raw);
      expect(tx.payee).toBeNull();
      expect(tx.subcategoryId).toBeUndefined();
      expect(tx.notes).toBeUndefined();
      expect(tx.status).toBe("pending");
      expect(tx.source).toBe("import");
    });

    it("handles null description as null", () => {
      const raw = {
        id: "tx-imported-303",
        source: "import",
        date: "2026-10-06",
        description: null,
        payee: "Shell Gas",
        amount: -5000,
        typeId: 2,
        accountId: 1,
        categoryId: 3,
        subcategoryId: null,
        notes: null,
        status: "cleared",
      };
      const tx = parseTransaction(raw);
      expect(tx.description).toBeNull();
    });

    it("throws ContractViolationError on missing fields or invalid types", () => {
      expect(() => parseTransaction(null)).toThrow(ContractViolationError);
      expect(() => parseTransaction("string")).toThrow(ContractViolationError);
      expect(() =>
        parseTransaction({
          id: "",
          source: "manual",
          date: "2026-10-05",
          description: "Test",
          amount: 100,
          typeId: 1,
          accountId: 1,
          categoryId: 1,
          status: "cleared",
        }),
      ).toThrow(ContractViolationError);

      // Non-integer amount
      expect(() =>
        parseTransaction({
          id: "tx-1",
          source: "manual",
          date: "2026-10-05",
          description: "Test",
          amount: 12.34,
          typeId: 1,
          accountId: 1,
          categoryId: 1,
          status: "cleared",
        }),
      ).toThrow(ContractViolationError);

      // Invalid date
      expect(() =>
        parseTransaction({
          id: "tx-1",
          source: "manual",
          date: "invalid-date",
          description: "Test",
          amount: 100,
          typeId: 1,
          accountId: 1,
          categoryId: 1,
          status: "cleared",
        }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parsePaginatedTransactions", () => {
    it("decodes paginated envelope with items array and metadata", () => {
      const rawPaginated = {
        items: [
          {
            id: "tx-101",
            source: "manual",
            date: "2026-10-05",
            description: "WHOLEFDS SOMA",
            payee: "Whole Foods",
            amount: -8420,
            typeId: 2,
            accountId: 1,
            categoryId: 3,
            subcategoryId: null,
            notes: null,
            status: "cleared",
          },
        ],
        totalCount: 1,
        page: 1,
        pageSize: 20,
        totalPages: 1,
      };

      const result: PaginatedTransactionsDto = parsePaginatedTransactions(rawPaginated);
      expect(result.items).toHaveLength(1);
      expect(result.items[0]?.id).toBe("tx-101");
      expect(result.totalCount).toBe(1);
      expect(result.page).toBe(1);
      expect(result.pageSize).toBe(20);
      expect(result.totalPages).toBe(1);
      expect(Object.isFrozen(result)).toBe(true);
      expect(Object.isFrozen(result.items)).toBe(true);
    });

    it("throws ContractViolationError when items is not an array", () => {
      expect(() =>
        parsePaginatedTransactions({
          items: "not-an-array",
          totalCount: 0,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        }),
      ).toThrow(ContractViolationError);
    });
  });
});
