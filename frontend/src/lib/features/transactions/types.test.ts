import { describe, expect, it } from "vitest";
import { ContractViolationError } from "$lib/api";
import { parseTransaction, parseTransactionsResponse } from "./types";
import { INITIAL_MOCK_TRANSACTIONS } from "./mock";

describe("Transactions Domain Decoders (types.ts)", () => {
  it("decodes a valid raw transaction into a frozen typed entity", () => {
    const raw = {
      id: 1,
      date: "2026-10-05",
      payee: "Whole Foods Market",
      amount: 8420,
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

    const decoded = parseTransaction(raw);
    expect(decoded.id).toBe(1);
    expect(decoded.description).toBe("Whole Foods Market");
    expect(decoded.payee).toBe("Whole Foods Market");
    expect(decoded.amount).toBe(8420);
    expect(decoded.typeId).toBe(2);
    expect(decoded.type).toBe("Expense");
    expect(decoded.typeColor).toBe("#f43f5e");
    expect(decoded.status).toBe("cleared");
    expect(Object.isFrozen(decoded)).toBe(true);

    const statementTx = parseTransaction({
      id: 2,
      date: "2026-10-05",
      description: "WHOLEFDS SOMA #10294",
      payee: "",
      amount: 8420,
      typeId: 2,
      type: "Expense",
      typeColor: "#f43f5e",
      memberId: 1,
      accountId: 3,
      categoryId: 2,
    });
    expect(statementTx.description).toBe("WHOLEFDS SOMA #10294");
    expect(statementTx.payee).toBe("");
  });

  it("throws ContractViolationError on missing or invalid fields", () => {
    // Non-object
    expect(() => parseTransaction(null)).toThrow(ContractViolationError);
    expect(() => parseTransaction("string")).toThrow(ContractViolationError);

    // Invalid date
    expect(() =>
      parseTransaction({
        id: 1,
        date: "invalid-date",
        payee: "Test",
        amount: 1000,
        typeId: 2,
        type: "Expense",
        memberId: 1,
        accountId: 1,
        categoryId: 1,
      }),
    ).toThrow(ContractViolationError);

    // Invalid amount (non-integer)
    expect(() =>
      parseTransaction({
        id: 1,
        date: "2026-10-05",
        payee: "Test",
        amount: 10.5,
        typeId: 2,
        type: "Expense",
        memberId: 1,
        accountId: 1,
        categoryId: 1,
      }),
    ).toThrow(ContractViolationError);

    // Invalid typeId (non-integer)
    expect(() =>
      parseTransaction({
        id: 1,
        date: "2026-10-05",
        payee: "Test",
        amount: 1000,
        typeId: "invalid",
        type: "Expense",
        memberId: 1,
        accountId: 1,
        categoryId: 1,
      }),
    ).toThrow(ContractViolationError);

    // Empty type string
    expect(() =>
      parseTransaction({
        id: 1,
        date: "2026-10-05",
        payee: "Test",
        amount: 1000,
        typeId: 2,
        type: "   ",
        memberId: 1,
        accountId: 1,
        categoryId: 1,
      }),
    ).toThrow(ContractViolationError);
  });

  it("decodes mock transactions array cleanly", () => {
    const parsed = parseTransactionsResponse(INITIAL_MOCK_TRANSACTIONS);
    expect(parsed.length).toBe(INITIAL_MOCK_TRANSACTIONS.length);
    expect(Object.isFrozen(parsed)).toBe(true);
  });
});
