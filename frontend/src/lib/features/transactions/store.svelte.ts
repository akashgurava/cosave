import { SvelteSet, SvelteMap } from "svelte/reactivity";
import { ApiError } from "$lib/api";
import type { AsyncState, MinorUnits, CurrencyId } from "$lib/types/core";
import { categoriesApi } from "$lib/features/categories/api";
import { categoryStore } from "$lib/features/categories/store";
import { familyApi } from "$lib/features/family/api";
import type { TransactionTypeItem } from "$lib/features/categories/types";
import type { Account, CurrencyOption, Member } from "$lib/features/family/types";
import { transactionsApi } from "./api";
import { INITIAL_MOCK_TRANSACTIONS } from "./mock";
import {
  type AccountId,
  type AmountPreset,
  type CategoryId,
  type CreateTransactionInput,
  type DatePreset,
  type MemberId,
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
  type UpdateTransactionInput,
} from "./types";

export interface TimelineGroup {
  readonly dateKey: string;
  readonly title: string;
  readonly net: MinorUnits;
  readonly items: readonly Transaction[];
}

export const AMOUNT_FIXED_POINTS = [0, 100, 500, 1000, 2000, Infinity] as const;
export const AMOUNT_POINT_LABELS = ["$0", "$100", "$500", "$1k", "$2k", "> $2k"] as const;

function isSameFieldValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a === undefined && b === undefined) return true;
  if (typeof a === "string" && typeof b === "string") {
    return a.trim() === b.trim();
  }
  return false;
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
    status: "success",
    data: Object.freeze([...INITIAL_MOCK_TRANSACTIONS]),
  });
  #transport: TransactionsTransport;

  // Metadata loaded live from configuration APIs
  #types = $state<readonly TransactionTypeItem[]>([]);
  #members = $state<readonly Member[]>([]);
  #accounts = $state<readonly Account[]>([]);
  #currencies = $state<readonly CurrencyOption[]>([]);
  #baseCurrency = $state<CurrencyOption>(DEFAULT_BASE_CURRENCY);

  #filters = $state<TransactionFilters>({ ...DEFAULT_TRANSACTION_FILTERS });
  #amountPointRange = $state<[number, number]>([0, 5]);

  #sortField = $state<SortField>("date");
  #sortDirection = $state<SortDirection>("desc");

  #collapsedGroupKeys = new SvelteSet<string>();
  #rowDrafts = $state<Record<TransactionId, Partial<Transaction>>>({});

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

  get types(): readonly TransactionTypeItem[] {
    if (categoryStore.types.length > 0) {
      return categoryStore.types;
    }
    return this.#types;
  }

  get members(): readonly Member[] {
    return this.#members;
  }

  get accounts(): readonly Account[] {
    return this.#accounts;
  }

  get currencies(): readonly CurrencyOption[] {
    return this.#currencies;
  }

  get baseCurrency(): CurrencyOption {
    return this.#baseCurrency;
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

  get rowDrafts(): Record<TransactionId, Partial<Transaction>> {
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
          const matchDesc = effective.description.toLowerCase().includes(query);
          const matchPayee = effective.payee.toLowerCase().includes(query);
          const matchNotes = effective.notes?.toLowerCase().includes(query) ?? false;
          if (!matchDesc && !matchPayee && !matchNotes) return false;
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
        if (f.selectedMemberIds.length > 0 && !f.selectedMemberIds.includes(effective.memberId)) {
          return false;
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
          case "payee":
            comparison = (effA.payee || "").localeCompare(effB.payee || "");
            break;
          case "description":
            comparison = effA.description.localeCompare(effB.description);
            break;
          case "type":
            comparison = effA.type.localeCompare(effB.type);
            break;
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
    const todayStr = "2026-10-05"; // Mock reference today
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

      // Financial net sum calculation: Income adds, Expense subtracts, Transfer/Invest neutral
      const normalizedType = eff.type.toLowerCase();
      if (normalizedType === "income") {
        g.net += Math.abs(eff.amount);
      } else if (normalizedType === "expense") {
        g.net -= Math.abs(eff.amount);
      }
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
    const [hierResult, famResult] = await Promise.allSettled([
      categoriesApi.getHierarchy(),
      familyApi.getDetails(),
    ]);

    if (hierResult.status === "fulfilled") {
      this.#types = Object.freeze(hierResult.value.types);
      if (categoryStore.types.length === 0) {
        void categoryStore.load();
      }
    }

    if (famResult.status === "fulfilled") {
      const details = famResult.value;
      this.#members = Object.freeze(details.members);
      this.#accounts = Object.freeze(details.accounts);
      this.#currencies = Object.freeze(details.currencies);
      const familyCurrencyId = details.family?.currencyId;
      const base =
        details.currencies.find((c) => c.id === familyCurrencyId) ??
        details.currencies[0] ??
        DEFAULT_BASE_CURRENCY;
      this.#baseCurrency = base;
    }
  }

  async load(queryFilters?: TransactionQueryFilters): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const data = await this.#transport.getTransactions(queryFilters);
      this.#state = { status: "success", data: Object.freeze([...data]) };
    } catch (err) {
      const action = err instanceof ApiError ? (err.action ?? "TX.LOAD.FAILED") : "TX.LOAD.FAILED";
      const message = err instanceof Error ? err.message : "Failed to load transactions";
      this.#state = { status: "error", error: { action, message } };
    }
  }

  async create(payload: CreateTransactionInput): Promise<Transaction> {
    const created = await this.#transport.createTransaction(payload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze([created, ...this.#state.data]),
      };
    }
    return created;
  }

  async update(id: TransactionId | number, payload: UpdateTransactionInput): Promise<Transaction> {
    const updated = await this.#transport.updateTransaction(id, payload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze(this.#state.data.map((t) => (t.id === id ? updated : t))),
      };
    }
    this.discardRowDraft(id as TransactionId);
    return updated;
  }

  async delete(id: TransactionId | number): Promise<void> {
    await this.#transport.deleteTransaction(id);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze(this.#state.data.filter((t) => t.id !== id)),
      };
    }
    this.discardRowDraft(id as TransactionId);
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
    if (!draft || Object.keys(draft).length === 0) return original;
    return Object.freeze({ ...original, ...draft });
  }

  hasRowDraft(id: TransactionId): boolean {
    const draft = this.#rowDrafts[id];
    if (!draft || Object.keys(draft).length === 0) return false;
    const original = this.transactions.find((t) => t.id === id);
    if (!original) return false;
    return Object.entries(draft).some(([k, v]) => {
      const key = k as keyof Transaction;
      return !isSameFieldValue(original[key], v);
    });
  }

  setRowDraftField<K extends keyof Transaction>(
    id: TransactionId,
    field: K,
    value: Transaction[K],
  ): void {
    this.setRowDraftFields(id, { [field]: value });
  }

  setRowDraftFields(id: TransactionId, updates: Partial<Transaction>): void {
    const original = this.transactions.find((t) => t.id === id);
    if (!original) return;

    const current = { ...(this.#rowDrafts[id] ?? {}) };

    for (const [key, value] of Object.entries(updates)) {
      const k = key as keyof Transaction;
      const origVal = original[k];
      if (isSameFieldValue(origVal, value)) {
        delete current[k];
      } else {
        (current as Record<string, unknown>)[k] = value;
      }
    }

    if (Object.keys(current).length === 0) {
      const next = { ...this.#rowDrafts };
      delete next[id];
      this.#rowDrafts = next;
    } else {
      this.#rowDrafts = { ...this.#rowDrafts, [id]: current };
    }
  }

  discardRowDraft(id: TransactionId): void {
    if (this.#rowDrafts[id]) {
      const next = { ...this.#rowDrafts };
      delete next[id];
      this.#rowDrafts = next;
    }
  }

  async saveRowDraft(id: TransactionId): Promise<Transaction | undefined> {
    const draft = this.#rowDrafts[id];
    if (!draft || Object.keys(draft).length === 0) return undefined;

    const payload: UpdateTransactionInput = {
      ...(draft.date !== undefined ? { date: draft.date } : {}),
      ...(draft.description !== undefined ? { description: draft.description } : {}),
      ...(draft.payee !== undefined ? { payee: draft.payee } : {}),
      ...(draft.amount !== undefined ? { amount: draft.amount } : {}),
      ...(draft.typeId !== undefined ? { typeId: draft.typeId } : {}),
      ...(draft.type !== undefined ? { type: draft.type } : {}),
      ...(draft.typeColor !== undefined ? { typeColor: draft.typeColor } : {}),
      ...(draft.memberId !== undefined ? { memberId: draft.memberId } : {}),
      ...(draft.accountId !== undefined ? { accountId: draft.accountId } : {}),
      ...(draft.categoryId !== undefined ? { categoryId: draft.categoryId } : {}),
      ...(draft.subcategoryId !== undefined ? { subcategoryId: draft.subcategoryId } : {}),
      ...(draft.notes !== undefined ? { notes: draft.notes } : {}),
      ...(draft.status !== undefined ? { status: draft.status } : {}),
    };

    return this.update(id, payload);
  }
}
