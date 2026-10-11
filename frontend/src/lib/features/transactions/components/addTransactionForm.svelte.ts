import { SvelteMap } from "svelte/reactivity";
import type { CategoryItem, TransactionTypeItem } from "$lib/features/categories/types";
import type { Member, Account } from "$lib/features/family/types";
import type { NewTransaction } from "../types";
import { getTodayIsoDate, parseCurrencyInput } from "../filters";
import type InputField from "$lib/components/InputField.svelte";
import {
  expectPresent,
  type MemberId,
  type AccountId,
  type CategoryId,
  type SubcategoryId,
  type TypeId,
} from "$lib/types";

export interface AddTransactionFormContext {
  getTypes: () => readonly TransactionTypeItem[];
  getMembers: () => readonly Member[];
  getAccounts: () => readonly Account[];
  onAddTransaction: (newTx: NewTransaction) => void;
  close: () => void;
}

export class AddTransactionForm {
  readonly #ctx: AddTransactionFormContext;

  date = $state(getTodayIsoDate());
  description = $state("");
  payee = $state("");
  amountStr = $state("");
  typeId = $state<TypeId | undefined>(undefined);
  memberId = $state<MemberId | undefined>(undefined);
  accountId = $state<AccountId | undefined>(undefined);
  categoryId = $state<CategoryId | undefined>(undefined);
  subcategoryId = $state<SubcategoryId | undefined>(undefined);
  error = $state<string | null>(null);
  amountField = $state<ReturnType<typeof InputField> | null>(null);

  constructor(ctx: AddTransactionFormContext) {
    this.#ctx = ctx;

    const initialTypes = this.#ctx.getTypes();
    const defaultType = initialTypes[0];
    if (defaultType !== undefined) {
      this.typeId = defaultType.id;
      const firstCat = defaultType.categories[0];
      if (firstCat !== undefined) {
        this.categoryId = firstCat.id;
        const sub = firstCat.subcategories[0];
        this.subcategoryId = sub !== undefined ? sub.id : undefined;
      }
    }

    const initialMembers = this.#ctx.getMembers();
    const firstMember = initialMembers[0];
    if (firstMember !== undefined) {
      this.memberId = firstMember.id;
      const initialAccounts = this.#ctx
        .getAccounts()
        .filter((a) => a.ownerMemberId === firstMember.id);
      const firstAcc = initialAccounts[0];
      if (firstAcc !== undefined) {
        this.accountId = firstAcc.id;
      }
    }

    // Initialize or align default type selection when types load
    $effect(() => {
      const types = this.#ctx.getTypes();
      if (
        types.length > 0 &&
        (this.typeId === undefined || !types.some((t) => t.id === this.typeId))
      ) {
        const t = types[0];
        if (t !== undefined) {
          this.typeId = t.id;
        }
      }
    });

    // Initialize or align default member selection when members load
    $effect(() => {
      const members = this.#ctx.getMembers();
      if (
        members.length > 0 &&
        (this.memberId === undefined || !members.some((m) => m.id === this.memberId))
      ) {
        const m = members[0];
        if (m !== undefined) {
          this.memberId = m.id;
        }
      }
    });

    // Update account when member changes if account doesn't belong to member
    $effect(() => {
      const accounts = this.modalAvailableAccounts;
      const firstAcc = accounts[0];
      if (
        firstAcc !== undefined &&
        (this.accountId === undefined || !accounts.some((a) => a.id === this.accountId))
      ) {
        this.accountId = firstAcc.id;
      } else if (accounts.length === 0) {
        this.accountId = undefined;
      }
    });

    // Update category when type changes if category doesn't belong to type
    $effect(() => {
      const categories = this.modalAvailableCategories;
      const firstCat = categories[0];
      if (
        firstCat !== undefined &&
        (this.categoryId === undefined || !categories.some((c) => c.id === this.categoryId))
      ) {
        this.categoryId = firstCat.id;
        const sub = firstCat.subcategories[0];
        this.subcategoryId = sub !== undefined ? sub.id : undefined;
      }
    });
  }

  get modalAvailableAccounts() {
    if (this.memberId === undefined) return [];
    return this.#ctx.getAccounts().filter((a) => a.ownerMemberId === this.memberId);
  }

  get typeMap() {
    return new SvelteMap(this.#ctx.getTypes().map((t) => [t.id, t]));
  }

  get categoryMap() {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of this.#ctx.getTypes()) {
      for (const c of t.categories) {
        map.set(c.id, c);
      }
    }
    return map;
  }

  get modalAvailableCategories() {
    if (this.typeId === undefined) return [];
    const selectedType = this.typeMap.get(this.typeId);
    return selectedType?.categories ?? [];
  }

  get modalAvailableSubcategories() {
    if (this.categoryId === undefined) return [];
    const selectedCat = this.categoryMap.get(this.categoryId);
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
    if (this.categoryId === undefined) {
      this.subcategoryId = undefined;
      return;
    }
    const cat = this.categoryMap.get(this.categoryId);
    const sub = cat?.subcategories[0];
    this.subcategoryId = sub !== undefined ? sub.id : undefined;
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

    if (this.modalAvailableAccounts.length === 0 || this.accountId === undefined) {
      this.error = "No account found for this member. Please add an account first.";
      return;
    }

    if (this.typeId === undefined) {
      this.error = "Transaction type is required.";
      return;
    }

    if (this.categoryId === undefined) {
      this.error = "Category is required.";
      return;
    }

    const currentType = expectPresent(
      this.typeMap.get(this.typeId),
      "VIEW.ADD_TRANSACTION_MODAL.RESOLVE_TYPE",
      `Transaction type ${this.typeId} not found in available types`,
    );

    this.#ctx.onAddTransaction({
      source: "manual",
      date: this.date,
      description: this.description.trim().length > 0 ? this.description.trim() : null,
      payee: this.payee.trim().length > 0 ? this.payee.trim() : null,
      amount: parsedAmount,
      typeId: currentType.id,
      accountId: this.accountId,
      categoryId: this.categoryId,
      subcategoryId: this.subcategoryId,
      status: "cleared",
    });

    this.description = "";
    this.payee = "";
    this.amountStr = "";
    this.#ctx.close();
  };
}
