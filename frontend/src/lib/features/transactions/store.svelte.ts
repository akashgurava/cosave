import { SvelteSet, SvelteMap } from "svelte/reactivity";
import { ApiError } from "$lib/api";
import { expectPresent, type AsyncState, type MinorUnits, type CurrencyId } from "$lib/types";
import { errorToToast } from "$lib/toast";
import { categoryStore } from "$lib/features/categories";
import { familyStore } from "$lib/features/family";
import type {
  CategoryItem,
  SubcategoryItem,
  TransactionTypeItem,
} from "$lib/features/categories/types";
import type { Account, CurrencyOption, Member } from "$lib/features/family/types";
import { transactionsApi } from "./api";
import {
  type AccountId,
  type AmountPreset,
  type CategoryId,
  type DatePreset,
  type MemberId,
  type NewTransaction,
  type SortDirection,
  type SortField,
  type SubcategoryId,
  type Transaction,
  type TransactionFilters,
  type TransactionId,
  type TransactionQueryFilters,
  type TransactionsTransport,
  type TransactionStatus,
  type TypeId,
} from "./types";

export interface TimelineGroup {
  readonly dateKey: string;
  readonly title: string;
  readonly net: MinorUnits;
  readonly items: readonly Transaction[];
}

export const AMOUNT_FIXED_POINTS = [0, 100, 500, 1000, 2000, Infinity] as const;
export const AMOUNT_POINT_LABELS = ["$0", "$100", "$500", "$1k", "$2k", "> $2k"] as const;

function isSameTransaction(a: Transaction, b: Transaction): boolean {
  return (
    a.date === b.date &&
    a.description === b.description &&
    a.payee === b.payee &&
    a.amount === b.amount &&
    a.typeId === b.typeId &&
    a.accountId === b.accountId &&
    a.categoryId === b.categoryId &&
    a.subcategoryId === b.subcategoryId &&
    a.notes === b.notes &&
    a.status === b.status
  );
}

export const DEFAULT_TRANSACTION_FILTERS: TransactionFilters = Object.freeze({
  searchQuery: "",
  datePreset: "all" as DatePreset,
  amountPreset: "all" as AmountPreset,
  selectedMemberIds: Object.freeze([]) as readonly MemberId[],
  selectedAccountIds: Object.freeze([]) as readonly AccountId[],
  selectedTypeIds: Object.freeze([]) as readonly TypeId[],
  selectedCategoryIds: Object.freeze([]) as readonly CategoryId[],
  selectedSubcategoryIds: Object.freeze([]) as readonly SubcategoryId[],
  selectedStatuses: Object.freeze([]) as readonly TransactionStatus[],
});

const DEFAULT_BASE_CURRENCY: CurrencyOption = Object.freeze({
  id: 1 as CurrencyId,
  code: "USD",
  name: "US Dollar",
  symbol: "$",
  scale: 2,
});

export class TransactionsStore {
  #state = $state<AsyncState<readonly Transaction[]>>({
    status: "idle",
  });
  #transport: TransactionsTransport;

  #totalCount = $state(0);
  #page = $state(1);
  #pageSize = $state(20);
  #totalPages = $state(1);

  #filters = $state<TransactionFilters>({ ...DEFAULT_TRANSACTION_FILTERS });
  #amountPointRange = $state<[number, number]>([0, 5]);

  #sortField = $state<SortField>("date");
  #sortDirection = $state<SortDirection>("desc");

  #collapsedGroupKeys = new SvelteSet<string>();
  #rowDrafts = $state<Record<TransactionId, Transaction>>({});

  constructor(transport: TransactionsTransport = transactionsApi) {
    this.#transport = transport;
  }

  // --- Getters ---
  get state(): AsyncState<readonly Transaction[]> {
    return this.#state;
  }

  get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  get isLoaded(): boolean {
    return this.#state.status === "success";
  }

  get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  get transactions(): readonly Transaction[] {
    return this.#state.status === "success" ? this.#state.data : [];
  }

  get totalCount(): number {
    return this.#totalCount;
  }

  get page(): number {
    return this.#page;
  }

  get pageSize(): number {
    return this.#pageSize;
  }

  get totalPages(): number {
    return this.#totalPages;
  }

  get types(): readonly TransactionTypeItem[] {
    return categoryStore.types;
  }

  get members(): readonly Member[] {
    return familyStore.members;
  }

  get accounts(): readonly Account[] {
    return familyStore.accounts;
  }

  get currencies(): readonly CurrencyOption[] {
    return familyStore.currencies;
  }

  // Reactive derived O(1) indices
  #transactionMap = $derived(new SvelteMap(this.transactions.map((t) => [t.id, t])));
  #typeMap = $derived(new SvelteMap(categoryStore.types.map((t) => [t.id, t])));
  #categoryMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of this.types) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, c);
      }
    }
    return map;
  });
  #subcategoryMap = $derived.by(() => {
    const map = new SvelteMap<SubcategoryId, SubcategoryItem>();
    for (const t of this.types) {
      for (const c of t.categories) {
        for (const s of c.subcategories) {
          map.set(s.id as SubcategoryId, s);
        }
      }
    }
    return map;
  });
  #typeByCategoryIdMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, TransactionTypeItem>();
    for (const t of this.types) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, t);
      }
    }
    return map;
  });

  // Invariant-asserting authoritative getters with unique SCREAMING action tokens
  getTransaction(id: TransactionId): Transaction {
    return expectPresent(
      this.#transactionMap.get(id),
      "STORE.TRANSACTION.GET_TRANSACTION",
      `Transaction ${id} not found in store`,
    );
  }

  getMember(id: MemberId): Member {
    return familyStore.requireMember(id);
  }

  getAccount(id: AccountId): Account {
    return familyStore.requireAccount(id);
  }

  getMemberIdForAccount(accountId: AccountId): MemberId {
    return this.getAccount(accountId).ownerMemberId;
  }

  getMemberForAccount(accountId: AccountId): Member {
    return this.getMember(this.getMemberIdForAccount(accountId));
  }

  getCurrency(id: CurrencyId): CurrencyOption {
    return familyStore.requireCurrency(id);
  }

  getType(id: TypeId | number): TransactionTypeItem {
    return expectPresent(
      this.#typeMap.get(id as TypeId),
      "STORE.TRANSACTION.GET_TYPE",
      `Type ${id} not found in transactions store`,
    );
  }

  getTypeName(typeId: TypeId): string {
    return this.getType(typeId).name;
  }

  getTypeColor(typeId: TypeId): string {
    return this.getType(typeId).color;
  }

  getCategory(id: CategoryId | number): CategoryItem {
    return expectPresent(
      this.#categoryMap.get(id as CategoryId),
      "STORE.TRANSACTION.GET_CATEGORY",
      `Category ${id} not found in transactions store`,
    );
  }

  getSubcategory(id: SubcategoryId | number): SubcategoryItem {
    return expectPresent(
      this.#subcategoryMap.get(id as SubcategoryId),
      "STORE.TRANSACTION.GET_SUBCATEGORY",
      `Subcategory ${id} not found in transactions store`,
    );
  }

  getTypeForCategory(catId: CategoryId | number): TransactionTypeItem {
    return expectPresent(
      this.#typeByCategoryIdMap.get(catId as CategoryId),
      "STORE.TRANSACTION.GET_TYPE_FOR_CATEGORY",
      `Parent Type for category ${catId} not found in transactions store`,
    );
  }

  get baseCurrency(): CurrencyOption {
    const cur = familyStore.currencies.find((c) => c.id === familyStore.currencyId);
    return cur ?? familyStore.currencies[0] ?? DEFAULT_BASE_CURRENCY;
  }

  get filters(): TransactionFilters {
    return this.#filters;
  }

  get amountPointRange(): [number, number] {
    return this.#amountPointRange;
  }

  set amountPointRange(range: [number, number]) {
    this.#amountPointRange = range;
  }

  get sortField(): SortField {
    return this.#sortField;
  }

  get sortDirection(): SortDirection {
    return this.#sortDirection;
  }

  get collapsedGroupKeys(): SvelteSet<string> {
    return this.#collapsedGroupKeys;
  }

  get rowDrafts(): Record<TransactionId, Transaction> {
    return this.#rowDrafts;
  }

  // --- Derived State & Computations ---
  get isAmountFiltered(): boolean {
    const low = this.#amountPointRange[0] ?? 0;
    const high = this.#amountPointRange[1] ?? 5;
    return low > 0 || high < 5;
  }

  get customAmountMin(): number | undefined {
    const low = this.#amountPointRange[0] ?? 0;
    if (low === 0) return undefined;
    const val = AMOUNT_FIXED_POINTS[low];
    if (val === undefined || !Number.isFinite(val)) {
      return 2000 * 100 + 1;
    }
    return val * 100;
  }

  get customAmountMax(): number | undefined {
    const high = this.#amountPointRange[1] ?? 5;
    if (high >= 5) return undefined;
    const val = AMOUNT_FIXED_POINTS[high];
    if (val === undefined || !Number.isFinite(val)) return undefined;
    return val * 100;
  }

  get amountDisplayLabel(): string {
    const low = this.#amountPointRange[0] ?? 0;
    const high = this.#amountPointRange[1] ?? 5;
    if (low === 0 && high === 5) return "$0 – > $2,000";
    if (low === 0) return `< ${AMOUNT_POINT_LABELS[high]}`;
    if (high === 5) {
      if (low === 5) return "> $2,000";
      return `≥ ${AMOUNT_POINT_LABELS[low]}`;
    }
    return `${AMOUNT_POINT_LABELS[low]} – ${AMOUNT_POINT_LABELS[high]}`;
  }

  get hasActiveFilters(): boolean {
    const f = this.#filters;
    return (
      f.searchQuery.trim().length > 0 ||
      f.datePreset !== "all" ||
      Boolean(f.customDateFrom) ||
      Boolean(f.customDateTo) ||
      this.isAmountFiltered ||
      f.selectedTypeIds.length > 0 ||
      f.selectedMemberIds.length > 0 ||
      f.selectedAccountIds.length > 0 ||
      f.selectedCategoryIds.length > 0 ||
      f.selectedSubcategoryIds.length > 0
    );
  }

  get filteredTransactions(): readonly Transaction[] {
    const txs = this.transactions;
    const f = this.#filters;
    const query = f.searchQuery.trim().toLowerCase();

    return txs
      .filter((tx) => {
        const effective = this.getEffectiveTx(tx);

        // Search query
        if (query.length > 0) {
          const matchDesc =
            effective.description !== null && effective.description.toLowerCase().includes(query);
          const matchPayee =
            effective.payee !== null && effective.payee.toLowerCase().includes(query);
          const matchNotes =
            effective.notes !== undefined &&
            effective.notes !== null &&
            effective.notes.toLowerCase().includes(query);
          if (matchDesc === false && matchPayee === false && matchNotes === false) return false;
        }

        // Date preset / range
        if (f.datePreset === "custom") {
          if (f.customDateFrom && effective.date < f.customDateFrom) return false;
          if (f.customDateTo && effective.date > f.customDateTo) return false;
        } else if (f.datePreset !== "all") {
          const txTime = Date.parse(effective.date);
          const nowTime = Date.parse("2026-10-06T00:00:00Z");
          const diffDays = (nowTime - txTime) / (1000 * 60 * 60 * 24);

          if (f.datePreset === "1d" && diffDays > 1) return false;
          if (f.datePreset === "3d" && diffDays > 3) return false;
          if (f.datePreset === "7d" && diffDays > 7) return false;
          if (f.datePreset === "1m" && diffDays > 30) return false;
          if (f.datePreset === "3m" && diffDays > 90) return false;
          if (f.datePreset === "6m" && diffDays > 180) return false;
          if (f.datePreset === "1y" && diffDays > 365) return false;
        }

        // Amount slider
        const absAmount = Math.abs(effective.amount);
        const min = this.customAmountMin;
        const max = this.customAmountMax;
        if (min !== undefined && absAmount < min) return false;
        if (max !== undefined && absAmount > max) return false;

        // Types (filter by typeId)
        if (f.selectedTypeIds.length > 0 && !f.selectedTypeIds.includes(effective.typeId)) {
          return false;
        }

        // Member
        if (f.selectedMemberIds.length > 0) {
          const memberId = this.getMemberIdForAccount(effective.accountId);
          if (!f.selectedMemberIds.includes(memberId)) {
            return false;
          }
        }

        // Account
        if (
          f.selectedAccountIds.length > 0 &&
          !f.selectedAccountIds.includes(effective.accountId)
        ) {
          return false;
        }

        // Category
        if (
          f.selectedCategoryIds.length > 0 &&
          !f.selectedCategoryIds.includes(effective.categoryId)
        ) {
          return false;
        }

        // Subcategory
        if (f.selectedSubcategoryIds.length > 0) {
          if (effective.subcategoryId) {
            if (!f.selectedSubcategoryIds.includes(effective.subcategoryId)) return false;
          } else {
            if (!f.selectedSubcategoryIds.includes(0 as SubcategoryId)) return false;
          }
        }

        return true;
      })
      .sort((a, b) => {
        const effA = this.getEffectiveTx(a);
        const effB = this.getEffectiveTx(b);
        let comparison: number;

        switch (this.#sortField) {
          case "date":
            comparison = effA.date.localeCompare(effB.date);
            break;
          case "amount":
            comparison = effA.amount - effB.amount;
            break;
          case "payee": {
            const pA = effA.payee !== null ? effA.payee : "";
            const pB = effB.payee !== null ? effB.payee : "";
            comparison = pA.localeCompare(pB);
            break;
          }
          case "description": {
            const dA = effA.description !== null ? effA.description : "";
            const dB = effB.description !== null ? effB.description : "";
            comparison = dA.localeCompare(dB);
            break;
          }
          case "type": {
            const tA = this.getTypeName(effA.typeId);
            const tB = this.getTypeName(effB.typeId);
            comparison = tA.localeCompare(tB);
            break;
          }
          case "status":
            comparison = effA.status.localeCompare(effB.status);
            break;
          default:
            comparison = effA.date.localeCompare(effB.date);
        }

        return this.#sortDirection === "asc" ? comparison : -comparison;
      });
  }

  get timelineGroups(): readonly TimelineGroup[] {
    const list = this.filteredTransactions;
    if (list.length === 0) return [];

    const nowTime = Date.parse("2026-10-06T00:00:00Z");
    const todayStr = "2026-10-05"; // Reference today
    const groupsMap = new SvelteMap<string, { title: string; items: Transaction[]; net: number }>();

    for (const tx of list) {
      const eff = this.getEffectiveTx(tx);
      let key = "older";
      let title = "Older";

      if (eff.date === todayStr) {
        key = "today";
        title = "Today";
      } else {
        const txTime = Date.parse(eff.date);
        const diffDays = Math.floor((nowTime - txTime) / (1000 * 60 * 60 * 24));

        if (diffDays <= 7) {
          key = "past-7-days";
          title = "Past 7 Days";
        } else if (diffDays <= 30) {
          key = "past-30-days";
          title = "Past 30 Days";
        } else if (eff.date.startsWith("2026-")) {
          const monthNames = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
          ];
          const monthNum = parseInt(eff.date.substring(5, 7), 10);
          const monthName = monthNames[monthNum - 1] ?? "Month";
          key = `month-${eff.date.substring(0, 7)}`;
          title = `${monthName} 2026`;
        }
      }

      if (!groupsMap.has(key)) {
        groupsMap.set(key, { title, items: [], net: 0 });
      }

      const g = groupsMap.get(key)!;
      g.items.push(tx);

      g.net += tx.amount;
    }

    return Object.freeze(
      Array.from(groupsMap.entries()).map(([dateKey, val]) =>
        Object.freeze({
          dateKey,
          title: val.title,
          net: val.net as MinorUnits,
          items: Object.freeze(val.items),
        }),
      ),
    );
  }

  // --- Actions ---

  async loadMetadata(): Promise<void> {
    await Promise.all([familyStore.load(), categoryStore.load()]);
  }

  async load(queryFilters?: TransactionQueryFilters): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const res = await this.#transport.getTransactions(queryFilters);
      this.#state = { status: "success", data: Object.freeze(res.items) };
      this.#totalCount = res.totalCount;
      this.#page = res.page;
      this.#pageSize = res.pageSize;
      this.#totalPages = res.totalPages;
    } catch (err) {
      const action = err instanceof ApiError ? (err.action ?? "TX.LOAD.FAILED") : "TX.LOAD.FAILED";
      const message = err instanceof Error ? err.message : "Failed to load transactions";
      this.#state = { status: "error", error: { action, message } };
    }
  }

  async create(payload: NewTransaction): Promise<Transaction> {
    try {
      const createdTx = await this.#transport.createTransaction(payload);

      if (this.#state.status === "success") {
        this.#state = {
          status: "success",
          data: Object.freeze([createdTx, ...this.#state.data]),
        };
        this.#totalCount += 1;
      }
      return createdTx;
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async update(transaction: Transaction): Promise<Transaction> {
    try {
      const updatedTx = await this.#transport.updateTransaction(transaction);

      if (this.#state.status === "success") {
        this.#state = {
          status: "success",
          data: Object.freeze(
            this.#state.data.map((t) => (t.id === transaction.id ? updatedTx : t)),
          ),
        };
      }
      this.discardRowDraft(transaction.id);
      return updatedTx;
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async delete(id: TransactionId): Promise<void> {
    try {
      const original = this.getTransaction(id);
      await this.#transport.deleteTransaction(id, original.source);

      if (this.#state.status === "success") {
        this.#state = {
          status: "success",
          data: Object.freeze(this.#state.data.filter((t) => t.id !== id)),
        };
        this.#totalCount = Math.max(0, this.#totalCount - 1);
      }
      this.discardRowDraft(id);
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  // --- Filtering & Sorting Methods ---

  setSort(field: SortField): void {
    if (this.#sortField === field) {
      this.#sortDirection = this.#sortDirection === "asc" ? "desc" : "asc";
    } else {
      this.#sortField = field;
      this.#sortDirection = "desc";
    }
  }

  toggleGroupCollapse(groupKey: string): void {
    if (this.#collapsedGroupKeys.has(groupKey)) {
      this.#collapsedGroupKeys.delete(groupKey);
    } else {
      this.#collapsedGroupKeys.add(groupKey);
    }
  }

  setFilter<K extends keyof TransactionFilters>(key: K, value: TransactionFilters[K]): void {
    this.#filters = { ...this.#filters, [key]: value };
  }

  resetFilters(): void {
    this.#filters = { ...DEFAULT_TRANSACTION_FILTERS };
    this.#amountPointRange = [0, 5];
  }

  // --- Row Drafts Management (In-Place Edit Staging) ---

  getEffectiveTx(original: Transaction): Transaction {
    const draft = this.#rowDrafts[original.id];
    return draft !== undefined ? draft : original;
  }

  hasRowDraft(id: TransactionId): boolean {
    return this.#rowDrafts[id] !== undefined;
  }

  setRowDraft(draft: Transaction): void {
    const original = this.#transactionMap.get(draft.id);
    if (original === undefined) return;
    if (isSameTransaction(original, draft)) {
      this.discardRowDraft(draft.id);
    } else {
      this.#rowDrafts = { ...this.#rowDrafts, [draft.id]: draft };
    }
  }

  discardRowDraft(id: TransactionId): void {
    if (this.#rowDrafts[id] !== undefined) {
      const next = { ...this.#rowDrafts };
      delete next[id];
      this.#rowDrafts = next;
    }
  }

  async saveRowDraft(id: TransactionId): Promise<Transaction> {
    const draft = expectPresent(
      this.#rowDrafts[id],
      "STORE.TRANSACTION.SAVE_ROW_DRAFT",
      `Row draft for transaction ${id} not found`,
    );
    return this.update(draft);
  }
}

export const transactionsStore = new TransactionsStore();
