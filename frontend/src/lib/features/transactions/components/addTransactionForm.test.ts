import { describe, it, expect, vi } from "vitest";
import { AddTransactionForm } from "./addTransactionForm.svelte";
import type { TransactionTypeItem } from "$lib/features/categories/types";
import type { Account, Member } from "$lib/features/family/types";
import type {
  AccountId,
  CategoryId,
  CurrencyId,
  FamilyId,
  MemberId,
  MinorUnits,
  SubcategoryId,
  TypeId,
} from "$lib/types";

describe("AddTransactionForm (Presentation Form Model)", () => {
  const mockTypes: readonly TransactionTypeItem[] = [
    {
      id: 2,
      name: "Expense",
      color: "#f43f5e",
      colorId: 1,
      categories: [
        {
          id: 1,
          name: "Groceries",
          subcategories: [
            {
              id: 10,
              name: "Supermarket",
            },
          ],
        },
      ],
    },
  ];

  const mockMembers: readonly Member[] = [
    {
      id: 1 as MemberId,
      familyId: 1 as FamilyId,
      memberName: "Alice",
      createdAt: 0,
    },
  ];

  const mockAccounts: readonly Account[] = [
    {
      id: 1 as AccountId,
      familyId: 1 as FamilyId,
      ownerMemberId: 1 as MemberId,
      type: "bank_account",
      bankName: "Chase",
      accountName: "Checking",
      last4: "1234",
      currencyId: 1 as CurrencyId,
      availableBalance: 100000 as MinorUnits,
      createdAt: 0,
    },
  ];

  function createForm(overrides?: {
    onAddTransaction?: (tx: any) => void;
    close?: () => void;
  }) {
    const onAddTransaction = overrides?.onAddTransaction ?? vi.fn();
    const close = overrides?.close ?? vi.fn();

    const form = new AddTransactionForm({
      getTypes: () => mockTypes,
      getMembers: () => mockMembers,
      getAccounts: () => mockAccounts,
      onAddTransaction,
      close,
    });

    return { form, onAddTransaction, close };
  }

  it("initializes with default date, type, category, and member", () => {
    const { form } = createForm();
    expect(form.date).toBe("2026-10-05");
    expect(form.description).toBe("");
    expect(form.payee).toBe("");
    expect(form.amountStr).toBe("");
    expect(form.error).toBeNull();
  });

  it("filters accounts belonging strictly to the selected member", () => {
    const { form } = createForm();
    expect(form.modalAvailableAccounts).toHaveLength(1);
    expect(form.modalAvailableAccounts[0]?.id).toBe(1 as AccountId);
  });

  it("rejects submission when amount is empty or non-numeric", () => {
    const { form, onAddTransaction } = createForm();
    form.amountStr = "";
    form.submit();
    expect(onAddTransaction).not.toHaveBeenCalled();

    form.amountStr = "abc";
    form.submit();
    expect(onAddTransaction).not.toHaveBeenCalled();
  });

  it("submits valid transaction with parsed minor units and direct description", () => {
    const { form, onAddTransaction, close } = createForm();
    form.amountStr = "42.50";
    form.accountId = 1 as AccountId;
    form.description = "Weekly shopping";
    form.payee = "Whole Foods";

    form.submit();

    expect(onAddTransaction).toHaveBeenCalledWith(
      expect.objectContaining({
        amount: 4250 as MinorUnits,
        description: "Weekly shopping",
        payee: "Whole Foods",
        type: "Expense",
        accountId: 1 as AccountId,
        categoryId: 1 as CategoryId,
      }),
    );
    expect(close).toHaveBeenCalled();
    expect(form.description).toBe("");
    expect(form.amountStr).toBe("");
  });

  it("passes empty description directly to delegating handler without throwing or falling back", () => {
    const { form, onAddTransaction, close } = createForm();
    form.amountStr = "15.00";
    form.accountId = 1 as AccountId;
    form.description = "";
    form.payee = "";

    form.submit();

    expect(onAddTransaction).toHaveBeenCalledWith(
      expect.objectContaining({
        amount: 1500 as MinorUnits,
        description: "",
        payee: "",
      }),
    );
    expect(close).toHaveBeenCalled();
  });
});
