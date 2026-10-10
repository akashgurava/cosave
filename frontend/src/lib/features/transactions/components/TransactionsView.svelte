<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { PlusIcon } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import type {
    Transaction,
    TransactionFilters,
    SortField,
    SortDirection,
    DatePreset,
    AmountPreset,
  } from "../types";
  import { applyFilters, resolveDatePresetToRange } from "../mock";
  import {
    expectPresent,
    type TransactionId,
    type MemberId,
    type AccountId,
    type CategoryId,
    type SubcategoryId,
    type TypeId,
    type MinorUnits,
  } from "$lib/types";
  import { TransactionsStore, type TimelineGroup } from "../store.svelte";
  import { familyStore } from "$lib/features/family";
  import { categoryStore } from "$lib/features/categories";
  import * as Pagination from "$lib/components/ui/pagination";
  import TransactionFilterBar from "./TransactionFilterBar.svelte";
  import TransactionTable from "./TransactionTable.svelte";
  import AddTransactionModal from "./AddTransactionModal.svelte";

  interface Props {
    transactions: readonly Transaction[];
    store?: TransactionsStore;
    onAddTransaction: (newTx: Omit<Transaction, "id">) => void;
    onUpdateTransaction: (id: TransactionId, updates: Partial<Transaction>) => void;
    onDeleteTransaction: (id: TransactionId) => void;
  }

  let {
    transactions,
    store = new TransactionsStore(),
    onAddTransaction,
    onUpdateTransaction,
    onDeleteTransaction,
  }: Props = $props();

  onMount(() => {
    void store.loadMetadata();
  });

  // Filter state
  let selectedMemberIds = $state<MemberId[]>([]);
  let selectedAccountIds = $state<AccountId[]>([]);
  let selectedTypeIds = $state<TypeId[]>([]);
  let selectedCategoryIds = $state<CategoryId[]>([]);
  let selectedSubcategoryIds = $state<SubcategoryId[]>([]);
  let datePreset = $state<DatePreset>("all");
  let customDateFrom = $state("");
  let customDateTo = $state("");
  let amountPointRange = $state<number[]>([0, 5]);
  let searchQuery = $state("");

  // Fixed-point amount points
  const AMOUNT_FIXED_POINTS = [0, 100, 500, 1000, 2000, Infinity] as const;

  // Sorting state
  let sortField = $state<SortField>("date");
  let sortDirection = $state<SortDirection>("desc");

  // Add Transaction Modal state
  let showAddModal = $state(false);

  // Row-level drafts for manual saving
  let rowDrafts = $state<Record<string, Partial<Transaction>>>({});

  const lowPointIdx = $derived(amountPointRange[0] ?? 0);
  const highPointIdx = $derived(amountPointRange[1] ?? 5);
  const isAmountFiltered = $derived(lowPointIdx > 0 || highPointIdx < 5);

  const customAmountMin = $derived.by<number | undefined>(() => {
    if (lowPointIdx === 0) return undefined;
    const val = AMOUNT_FIXED_POINTS[lowPointIdx];
    if (val === undefined || !Number.isFinite(val)) {
      return 2000 * 100 + 1;
    }
    return val * 100;
  });

  const customAmountMax = $derived.by<number | undefined>(() => {
    if (highPointIdx >= 5) return undefined;
    const val = AMOUNT_FIXED_POINTS[highPointIdx];
    if (val === undefined || !Number.isFinite(val)) return undefined;
    return val * 100;
  });

  const amountPreset = $derived<AmountPreset>(isAmountFiltered ? "custom" : "all");

  const resolvedDateRange = $derived.by(() => {
    if (datePreset === "custom") {
      return {
        startDate: customDateFrom || undefined,
        endDate: customDateTo || undefined,
      };
    }
    return resolveDatePresetToRange(datePreset, "2026-10-05");
  });

  const activeFilters = $derived<TransactionFilters>({
    searchQuery,
    datePreset,
    startDate: resolvedDateRange.startDate,
    endDate: resolvedDateRange.endDate,
    customDateFrom: customDateFrom || undefined,
    customDateTo: customDateTo || undefined,
    amountPreset,
    customAmountMin,
    customAmountMax,
    selectedTypeIds,
    selectedMemberIds,
    selectedAccountIds,
    selectedCategoryIds,
    selectedSubcategoryIds,
    selectedStatuses: [],
  });

  function resetAllFilters() {
    searchQuery = "";
    datePreset = "all";
    customDateFrom = "";
    customDateTo = "";
    amountPointRange = [0, 5];
    selectedTypeIds = [];
    selectedMemberIds = [];
    selectedAccountIds = [];
    selectedCategoryIds = [];
    selectedSubcategoryIds = [];
  }

  function handleSort(field: SortField) {
    if (sortField === field) {
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      sortField = field;
      sortDirection = "desc";
    }
  }

  function getEffectiveTx(tx: Transaction): Transaction {
    const draft = rowDrafts[tx.id];
    return draft ? { ...tx, ...draft } : tx;
  }

  const transactionMap = $derived(new Map(transactions.map((t) => [t.id, t])));

  function getOriginalTx(id: TransactionId): Transaction {
    return expectPresent(
      transactionMap.get(id),
      "VIEW.TRANSACTIONS_VIEW.GET_ORIGINAL_TX",
      `Transaction ${id} not found in transactions dataset`,
    );
  }

  function isSameFieldValue(a: unknown, b: unknown): boolean {
    if (a === b) return true;
    if ((a === null || a === undefined) && (b === null || b === undefined)) return true;
    if (typeof a === "string" || typeof b === "string") {
      const strA = typeof a === "string" ? a.trim() : "";
      const strB = typeof b === "string" ? b.trim() : "";
      return strA === strB;
    }
    return false;
  }

  function handleDraftChange(id: TransactionId, updates: Partial<Transaction>) {
    const original = getOriginalTx(id);
    if (!original) return;

    const current = { ...(rowDrafts[id] ?? {}) };

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
      const updated = { ...rowDrafts };
      delete updated[id];
      rowDrafts = updated;
    } else {
      rowDrafts = { ...rowDrafts, [id]: current };
    }
  }

  function hasRowDraft(id: TransactionId): boolean {
    const draft = rowDrafts[id];
    if (!draft || Object.keys(draft).length === 0) return false;
    const original = getOriginalTx(id);
    if (!original) return false;
    return Object.entries(draft).some(([k, v]) => {
      const key = k as keyof Transaction;
      return !isSameFieldValue(original[key], v);
    });
  }

  function handleSaveRowDraft(id: TransactionId) {
    const draft = rowDrafts[id];
    if (draft && Object.keys(draft).length > 0) {
      onUpdateTransaction(id, draft);
      const updated = { ...rowDrafts };
      delete updated[id];
      rowDrafts = updated;
    }
  }

  function handleDiscardRowDraft(id: TransactionId) {
    const updated = { ...rowDrafts };
    delete updated[id];
    rowDrafts = updated;
  }

  const filteredTransactions = $derived.by(() => {
    const list = applyFilters(transactions, activeFilters);
    return [...list].sort((a, b) => {
      let comparison: number;
      switch (sortField) {
        case "date":
          comparison = a.date.localeCompare(b.date);
          break;
        case "description":
          comparison = (a.description ?? "").localeCompare(b.description ?? "");
          break;
        case "payee":
          comparison = a.payee.localeCompare(b.payee);
          break;
        case "amount":
          comparison = a.amount - b.amount;
          break;
        case "type":
          comparison = a.type.localeCompare(b.type);
          break;
        case "member": {
          const ma = store.getMember(a.memberId).memberName;
          const mb = store.getMember(b.memberId).memberName;
          comparison = ma.localeCompare(mb);
          break;
        }
        case "account": {
          const accA = store.getAccount(a.accountId);
          const accB = store.getAccount(b.accountId);
          const aa = accA.type === "bank_account" ? accA.accountName : accA.cardName;
          const ab = accB.type === "bank_account" ? accB.accountName : accB.cardName;
          comparison = aa.localeCompare(ab);
          break;
        }
        case "category": {
          const ca = store.getCategory(a.categoryId).name;
          const cb = store.getCategory(b.categoryId).name;
          comparison = ca.localeCompare(cb);
          break;
        }
        default:
          comparison = 0;
      }
      return sortDirection === "asc" ? comparison : -comparison;
    });
  });

  // Pagination state (20 items per page limit)
  const pageSize = 20;
  let currentPage = $state(1);

  const totalFilteredCount = $derived(filteredTransactions.length);
  const totalPages = $derived(Math.max(1, Math.ceil(totalFilteredCount / pageSize)));

  const paginatedTransactions = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredTransactions.slice(start, start + pageSize);
  });

  const rangeStart = $derived(totalFilteredCount === 0 ? 0 : (currentPage - 1) * pageSize + 1);
  const rangeEnd = $derived(Math.min(currentPage * pageSize, totalFilteredCount));

  let previousFilterSignature = $state("");
  $effect(() => {
    const signature = `${searchQuery}|${datePreset}|${customDateFrom}|${customDateTo}|${amountPointRange.join(",")}|${selectedTypeIds.join(",")}|${selectedMemberIds.join(",")}|${selectedAccountIds.join(",")}|${selectedCategoryIds.join(",")}|${selectedSubcategoryIds.join(",")}`;
    if (previousFilterSignature !== "" && previousFilterSignature !== signature) {
      currentPage = 1;
    }
    previousFilterSignature = signature;
  });

  $effect(() => {
    if (currentPage > totalPages && totalPages > 0) {
      currentPage = totalPages;
    }
  });

  const timelineGroups = $derived.by<readonly TimelineGroup[]>(() => {
    const referenceToday = "2026-10-05";
    const refTime = new Date(`${referenceToday}T00:00:00Z`).getTime();

    const oneDayAgoStr = new Date(refTime - 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
    const sevenDaysAgoStr = new Date(refTime - 7 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
    const refMonthStr = referenceToday.slice(0, 7);

    const groupsMap = new SvelteMap<string, { order: number; items: Transaction[] }>();

    for (const tx of paginatedTransactions) {
      const eff = getEffectiveTx(tx);
      let groupKey: string;
      let order: number;

      if (eff.date === referenceToday) {
        groupKey = "Today";
        order = 1;
      } else if (eff.date >= sevenDaysAgoStr && eff.date <= oneDayAgoStr) {
        groupKey = "Past 7 Days";
        order = 2;
      } else if (eff.date.slice(0, 7) === refMonthStr) {
        groupKey = "Earlier This Month";
        order = 3;
      } else {
        const [yearStr, monthStr] = eff.date.split("-");
        const year = Number(yearStr);
        const month = Number(monthStr);
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
        const monthName = monthNames[(month || 1) - 1] ?? "Unknown";
        groupKey = `${monthName} ${year}`;
        order = 1000000 - (year * 100 + month);
      }

      const existing = groupsMap.get(groupKey) ?? { order, items: [] };
      existing.items.push(tx);
      groupsMap.set(groupKey, existing);
    }

    const result: TimelineGroup[] = [];
    for (const [title, entry] of groupsMap.entries()) {
      let net = 0;
      for (const item of entry.items) {
        const eff = getEffectiveTx(item);
        const normalized = eff.type.toLowerCase();
        if (normalized === "income") net += eff.amount;
        if (normalized === "expense") net -= eff.amount;
      }
      result.push({
        title,
        dateKey: title,
        items: entry.items,
        net: net as MinorUnits,
      });
    }

    return result;
  });
</script>

{#if familyStore.isLoading || categoryStore.isLoading}
  <div class="flex min-h-[40vh] items-center justify-center">
    <div
      class="border-border/40 size-8 animate-spin rounded-full border-2 border-t-emerald-500"
    ></div>
  </div>
{:else if store.members.length === 0}
  <div
    class="border-border/60 mx-auto flex max-w-xl flex-col items-center justify-center rounded-2xl border border-dashed p-12 text-center"
  >
    <div
      class="bg-muted/40 text-muted-foreground/80 border-border/40 mb-4 flex size-12 items-center justify-center rounded-xl border"
    >
      <PlusIcon class="size-6" />
    </div>
    <h3 class="text-foreground text-lg font-semibold tracking-tight">No family members found</h3>
    <p class="text-muted-foreground mt-1 max-w-sm text-sm">
      Add your first family member and account in Family & Accounts to start recording and
      categorizing transactions.
    </p>
    <Button
      href="/configuration/family"
      class="bg-foreground text-background hover:bg-foreground/90 mt-5"
    >
      Go to Family & Accounts
    </Button>
  </div>
{:else}
  <div class="mx-auto max-w-7xl space-y-4">
    <!-- Top Navigation & Prominent Monochromatic Button Bar -->
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-foreground text-xl font-semibold tracking-tight">Transactions</h2>
      </div>

      <div class="flex items-center gap-3">
        <Button
          size="sm"
          class="bg-foreground text-background hover:bg-foreground/90 h-9 gap-1.5 px-3.5 font-medium shadow-xs"
          onclick={() => (showAddModal = true)}
        >
          <PlusIcon class="size-4" />
          Add Transaction
        </Button>
      </div>
    </div>

    <!-- 3-Level Filter Panel -->
    <TransactionFilterBar
      types={store.types}
      members={store.members}
      accounts={store.accounts}
      bind:searchQuery
      bind:datePreset
      bind:customDateFrom
      bind:customDateTo
      bind:amountPointRange
      bind:selectedMemberIds
      bind:selectedAccountIds
      bind:selectedTypeIds
      bind:selectedCategoryIds
      bind:selectedSubcategoryIds
      onResetAll={resetAllFilters}
    />

    <!-- Unified Ledger Table with Sticky Timeline Groupings -->
    <TransactionTable
      {timelineGroups}
      {sortField}
      {sortDirection}
      types={store.types}
      members={store.members}
      accounts={store.accounts}
      currencies={store.currencies}
      baseCurrency={store.baseCurrency}
      onSort={handleSort}
      {rowDrafts}
      {hasRowDraft}
      onSaveRowDraft={handleSaveRowDraft}
      onDiscardRowDraft={handleDiscardRowDraft}
      {onDeleteTransaction}
      onDraftChange={handleDraftChange}
    />

    <!-- Pagination Footer -->
    {#if totalFilteredCount > 0}
      <div
        class="border-border/40 grid grid-cols-1 items-center gap-3 border-t pt-4 sm:grid-cols-3"
      >
        <!-- Left spacer to maintain perfect symmetry for centering the middle column -->
        <div class="hidden sm:block"></div>

        <!-- Center column: Paginator strictly centered -->
        <div class="flex justify-center">
          <Pagination.Root
            count={totalFilteredCount}
            perPage={pageSize}
            bind:page={currentPage}
            siblingCount={1}
            class="mx-0 w-auto"
          >
            {#snippet children({ pages })}
              <Pagination.Content>
                <Pagination.Item>
                  <Pagination.Previous />
                </Pagination.Item>
                {#each pages as page (page.key)}
                  {#if page.type === "ellipsis"}
                    <Pagination.Item>
                      <Pagination.Ellipsis />
                    </Pagination.Item>
                  {:else}
                    <Pagination.Item>
                      <Pagination.Link {page} isActive={currentPage === page.value}>
                        {page.value}
                      </Pagination.Link>
                    </Pagination.Item>
                  {/if}
                {/each}
                <Pagination.Item>
                  <Pagination.Next />
                </Pagination.Item>
              </Pagination.Content>
            {/snippet}
          </Pagination.Root>
        </div>

        <!-- Right column: Summary line all in 1 line at the right end of the table -->
        <div class="flex justify-center sm:justify-end">
          <p class="text-muted-foreground font-mono text-xs whitespace-nowrap">
            Showing <span class="text-foreground font-medium">{rangeStart}</span>–<span
              class="text-foreground font-medium">{rangeEnd}</span
            >
            of <span class="text-foreground font-medium">{totalFilteredCount}</span> transactions
          </p>
        </div>
      </div>
    {/if}
  </div>

  <!-- Monochromatic Add Transaction Dialog Modal -->
  <AddTransactionModal
    bind:open={showAddModal}
    types={store.types}
    members={store.members}
    accounts={store.accounts}
    currency={store.baseCurrency}
    {onAddTransaction}
  />
{/if}
