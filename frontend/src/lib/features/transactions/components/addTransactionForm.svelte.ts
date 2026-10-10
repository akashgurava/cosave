import { SvelteMap } from "svelte/reactivity";
import type { CategoryItem, TransactionTypeItem } from "$lib/features/categories/types";
import type { Member, Account } from "$lib/features/family/types";
import type { Transaction } from "../types";
import { parseCurrencyInput } from "../filters";
import type InputField from "$lib/components/InputField.svelte";
import {
  expectPresent,
  type MemberId,
  type AccountId,
  type CategoryId,
  type SubcategoryId,
  type MinorUnits,
  type TypeId,
} from "$lib/types";

export interface AddTransactionFormContext {
  getTypes: () => readonly TransactionTypeItem[];
  getMembers: () => readonly Member[];
  getAccounts: () => readonly Account[];
  onAddTransaction: (newTx: Omit<Transaction, "id">) => void;
  close: () => void;
}

export class AddTransactionForm {
  readonly #ctx: AddTransactionFormContext;

  date = $state("2026-10-05");
  description = $state("");
  payee = $state("");
  amountStr = $state("");
  typeId = $state<TypeId | number>(2 as TypeId);
  memberId = $state<MemberId | number>(1 as MemberId);
  accountId = $state<AccountId | number | undefined>(undefined);
  categoryId = $state<CategoryId | number>(1 as CategoryId);
  subcategoryId = $state<SubcategoryId | number | undefined>(undefined);
  error = $state<string | null>(null);
  amountField = $state<ReturnType<typeof InputField> | null>(null);

  constructor(ctx: AddTransactionFormContext) {
    this.#ctx = ctx;

    // Initialize or align default type selection when types load
    $effect(() => {
      const types = this.#ctx.getTypes();
      if (types.length > 0 && !types.some((t) => t.id === this.typeId)) {
        const defaultType = types[0];
        if (defaultType) {
          this.typeId = defaultType.id;
        }
      }
    });

    // Initialize or align default member selection when members load
    $effect(() => {
      const members = this.#ctx.getMembers();
      if (members.length > 0 && !members.some((m) => m.id === this.memberId)) {
        const firstMember = members[0];
        if (firstMember) {
          this.memberId = firstMember.id;
        }
      }
    });

    // Update account when member changes if account doesn't belong to member
    $effect(() => {
      const accounts = this.modalAvailableAccounts;
      if (accounts[0] && (!this.accountId || !accounts.some((a) => a.id === this.accountId))) {
        this.accountId = accounts[0].id;
      } else if (accounts.length === 0) {
        this.accountId = undefined;
      }
    });

    // Update category when type changes if category doesn't belong to type
    $effect(() => {
      const categories = this.modalAvailableCategories;
      if (categories[0] && !categories.some((c) => c.id === this.categoryId)) {
        this.categoryId = categories[0].id;
        const sub = categories[0].subcategories[0];
        this.subcategoryId = sub ? sub.id : undefined;
      }
    });
  }

  get modalAvailableAccounts() {
    return this.#ctx.getAccounts().filter((a) => a.ownerMemberId === this.memberId);
  }

  get typeMap() {
    return new SvelteMap(this.#ctx.getTypes().map((t) => [t.id, t]));
  }

  get categoryMap() {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of this.#ctx.getTypes()) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, c);
      }
    }
    return map;
  }

  get modalAvailableCategories() {
    const selectedType = this.typeMap.get(this.typeId as TypeId);
    return selectedType?.categories ?? [];
  }

  get modalAvailableSubcategories() {
    const selectedCat = this.categoryMap.get(this.categoryId as CategoryId);
    return selectedCat?.subcategories ?? [];
  }

  get memberOptions() {
    return this.#ctx.getMembers().map((m) => ({ value: m.id, label: m.memberName }));
  }

  get accountOptions() {
    return this.modalAvailableAccounts.map((acc) => ({
      value: acc.id,
      label: acc.type === "bank_account" ? acc.accountName : acc.cardName,
      sublabel: `${acc.bankName} ····${acc.last4}`,
    }));
  }

  get typeOptions() {
    return this.#ctx.getTypes().map((t) => ({ value: t.id, label: t.name, color: t.color }));
  }

  get categoryOptions() {
    return this.modalAvailableCategories.map((c) => ({ value: c.id, label: c.name }));
  }

  get subcategoryOptions() {
    return this.modalAvailableSubcategories.map((s) => ({ value: s.id, label: s.name }));
  }

  handleCategoryChange = () => {
    const cat = this.categoryMap.get(this.categoryId as CategoryId);
    const sub = cat?.subcategories[0];
    this.subcategoryId = sub ? sub.id : undefined;
  };

  submit = () => {
    this.error = null;

    if (this.amountField !== null && this.amountField.validate() === false) {
      return;
    }

    const parsedAmount = parseCurrencyInput(this.amountStr);
    if (parsedAmount === null || parsedAmount <= 0) {
      this.amountField?.shake();
      return;
    }

    if (!this.date) {
      this.error = "Transaction date is required.";
      return;
    }

    if (this.modalAvailableAccounts.length === 0 || !this.accountId) {
      this.error = "No account found for this member. Please add an account first.";
      return;
    }

    const currentType = expectPresent(
      this.typeMap.get(this.typeId as TypeId),
      "VIEW.ADD_TRANSACTION_MODAL.RESOLVE_TYPE",
      `Transaction type ${this.typeId} not found in available types`,
    );

    this.#ctx.onAddTransaction({
      source: "manual",
      date: this.date,
      description: this.description,
      payee: this.payee,
      amount: parsedAmount as MinorUnits,
      typeId: currentType.id as TypeId,
      type: currentType.name,
      typeColor: currentType.color,
      memberId: this.memberId as MemberId,
      accountId: this.accountId as AccountId,
      categoryId: this.categoryId as CategoryId,
      subcategoryId:
        this.subcategoryId !== undefined && Number(this.subcategoryId) > 0
          ? (this.subcategoryId as SubcategoryId)
          : undefined,
      status: "cleared",
    });

    this.description = "";
    this.payee = "";
    this.amountStr = "";
    this.#ctx.close();
  };
}
