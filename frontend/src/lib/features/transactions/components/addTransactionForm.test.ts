import { describe, it, expect, vi } from "vitest";
import { AddTransactionForm } from "./addTransactionForm.svelte";
import type { TransactionTypeItem } from "$lib/features/categories/types";
import type { Account, Member } from "$lib/features/family/types";
import {
  AccountId,
  CategoryId,
  ColorId,
  CurrencyId,
  FamilyId,
  MemberId,
  MinorUnits,
  SubcategoryId,
  TypeId,
} from "$lib/types";
import type { NewTransaction } from "../types";
import { getTodayIsoDate } from "../filters";

describe("AddTransactionForm (Presentation Form Model)", () => {
  const mockTypes: readonly TransactionTypeItem[] = [
    {
      id: TypeId(2),
      name: "Expense",
      color: "#f43f5e",
      colorId: ColorId(1),
      categories: [
        {
          id: CategoryId(1),
          name: "Groceries",
          subcategories: [
            {
              id: SubcategoryId(10),
              name: "Supermarket",
            },
          ],
        },
      ],
    },
  ];

  const mockMembers: readonly Member[] = [
    {
      id: MemberId(1),
      familyId: FamilyId(1),
      name: "Alice",
      createdAt: 0,
    },
  ];

  const mockAccounts: readonly Account[] = [
    {
      id: AccountId(1),
      familyId: FamilyId(1),
      ownerMemberId: MemberId(1),
      type: "bank_account",
      name: "Checking",
      currencyId: CurrencyId(1),
      availableBalance: MinorUnits(100000),
      createdAt: 0,
    },
  ];

  function createForm(overrides?: {
    onAddTransaction?: (tx: NewTransaction) => void;
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
    expect(form.date).toBe(getTodayIsoDate());
    expect(form.description).toBe("");
    expect(form.payee).toBe("");
    expect(form.amountStr).toBe("");
    expect(form.error).toBeNull();
  });

  it("filters accounts belonging strictly to the selected member", () => {
    const { form } = createForm();
    expect(form.modalAvailableAccounts).toHaveLength(1);
    expect(form.modalAvailableAccounts[0]?.id).toBe(AccountId(1));
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
    form.accountId = AccountId(1);
    form.description = "Weekly shopping";
    form.payee = "Whole Foods";

    form.submit();

    expect(onAddTransaction).toHaveBeenCalledWith(
      expect.objectContaining({
        amount: MinorUnits(4250),
        description: "Weekly shopping",
        payee: "Whole Foods",
        typeId: TypeId(2),
        accountId: AccountId(1),
        categoryId: CategoryId(1),
      }),
    );
    expect(close).toHaveBeenCalled();
    expect(form.description).toBe("");
    expect(form.amountStr).toBe("");
  });

  it("passes empty description directly to delegating handler as null without throwing or falling back", () => {
    const { form, onAddTransaction, close } = createForm();
    form.amountStr = "15.00";
    form.accountId = AccountId(1);
    form.description = "";
    form.payee = "";

    form.submit();

    expect(onAddTransaction).toHaveBeenCalledWith(
      expect.objectContaining({
        amount: MinorUnits(1500),
        description: null,
        payee: null,
      }),
    );
    expect(close).toHaveBeenCalled();
  });
});
