import { describe, expect, it } from "vitest";
import { ContractViolationError } from "$lib/api";
import {
  parsePaginatedTransactionsWireDto,
  parseTransaction,
  parseTransactionWireDto,
  type PaginatedTransactionsWireDto,
  type Transaction,
  type TransactionWireDto,
} from "./types";

describe("Transactions Wire & Domain Decoders (types.ts)", () => {
  describe("parseTransactionWireDto", () => {
    it("decodes valid backend wire DTO into frozen object", () => {
      const rawWire = {
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

      const dto: TransactionWireDto = parseTransactionWireDto(rawWire);
      expect(dto.id).toBe("tx-manual-101");
      expect(dto.source).toBe("manual");
      expect(dto.date).toBe("2026-10-05");
      expect(dto.description).toBe("WHOLEFDS SOMA #10294");
      expect(dto.payee).toBe("Whole Foods Market");
      expect(dto.amount).toBe(-8420);
      expect(dto.typeId).toBe(2);
      expect(dto.accountId).toBe(1);
      expect(dto.categoryId).toBe(3);
      expect(dto.subcategoryId).toBe(10);
      expect(dto.notes).toBe("Organic groceries");
      expect(dto.status).toBe("cleared");
      expect(Object.isFrozen(dto)).toBe(true);
    });

    it("handles null payee, null subcategoryId, and null notes", () => {
      const rawWire = {
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

      const dto = parseTransactionWireDto(rawWire);
      expect(dto.payee).toBeNull();
      expect(dto.subcategoryId).toBeNull();
      expect(dto.notes).toBeNull();
      expect(dto.status).toBe("pending");
      expect(dto.source).toBe("import");
    });

    it("handles null description as null", () => {
      const rawWire = {
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
      const dto = parseTransactionWireDto(rawWire);
      expect(dto.description).toBeNull();
    });

    it("throws ContractViolationError on missing fields or invalid types", () => {
      expect(() => parseTransactionWireDto(null)).toThrow(ContractViolationError);
      expect(() => parseTransactionWireDto("string")).toThrow(ContractViolationError);
      expect(() =>
        parseTransactionWireDto({
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
        parseTransactionWireDto({
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
        parseTransactionWireDto({
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

  describe("parsePaginatedTransactionsWireDto", () => {
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

      const result: PaginatedTransactionsWireDto = parsePaginatedTransactionsWireDto(rawPaginated);
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
        parsePaginatedTransactionsWireDto({
          items: "not-an-array",
          totalCount: 0,
          page: 1,
          pageSize: 20,
          totalPages: 1,
        }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parseTransaction (Presentation Model)", () => {
    it("decodes presentation transaction entity", () => {
      const raw = {
        id: "tx-101",
        source: "manual",
        date: "2026-10-05",
        description: "Whole Foods Market",
        payee: "Whole Foods Market",
        amount: -8420,
        typeId: 2,
        type: "Expense",
        typeColor: "#f43f5e",
        memberId: 1,
        accountId: 3,
        categoryId: 2,
        subcategoryId: 201,
        notes: "Weekly organic produce & dairy",
        status: "cleared",
      };

      const decoded: Transaction = parseTransaction(raw);
      expect(decoded.id).toBe("tx-101");
      expect(decoded.source).toBe("manual");
      expect(decoded.description).toBe("Whole Foods Market");
      expect(decoded.payee).toBe("Whole Foods Market");
      expect(decoded.amount).toBe(-8420);
      expect(decoded.typeId).toBe(2);
      expect(decoded.type).toBe("Expense");
      expect(decoded.typeColor).toBe("#f43f5e");
      expect(decoded.memberId).toBe(1);
      expect(decoded.accountId).toBe(3);
      expect(decoded.status).toBe("cleared");
      expect(Object.isFrozen(decoded)).toBe(true);
    });

    it("decodes presentation transaction entity with null description", () => {
      const raw = {
        id: "tx-102",
        source: "manual",
        date: "2026-10-05",
        description: null,
        payee: "Whole Foods Market",
        amount: -8420,
        typeId: 2,
        type: "Expense",
        typeColor: "#f43f5e",
        memberId: 1,
        accountId: 3,
        categoryId: 2,
        status: "cleared",
      };

      const decoded = parseTransaction(raw);
      expect(decoded.description).toBeNull();
    });
  });
});
