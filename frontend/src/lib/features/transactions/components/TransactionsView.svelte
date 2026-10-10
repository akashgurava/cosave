<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { PlusIcon } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { PaginationFooter } from "$lib/components";
  import type {
    Transaction,
    TransactionFilters,
    SortField,
    SortDirection,
    DatePreset,
    AmountPreset,
  } from "../types";
  import { applyFilters, resolveDatePresetToRange } from "../filters";
  import type {
    TransactionId,
    MemberId,
    AccountId,
    CategoryId,
    SubcategoryId,
    TypeId,
    MinorUnits,
  } from "$lib/types";
  import { TransactionsStore, transactionsStore, type TimelineGroup } from "../store.svelte";
  import { familyStore } from "$lib/features/family";
  import { categoryStore } from "$lib/features/categories";
  import TransactionFilterBar from "./TransactionFilterBar.svelte";
  import TransactionTable from "./TransactionTable.svelte";
  import AddTransactionModal from "./AddTransactionModal.svelte";

  interface Props {
    transactions?: readonly Transaction[];
    store?: TransactionsStore;
    onAddTransaction?: (newTx: Omit<Transaction, "id">) => void;
    onUpdateTransaction?: (id: TransactionId, updates: Partial<Transaction>) => void;
    onDeleteTransaction?: (id: TransactionId) => void;
  }

  let {
    transactions: explicitTransactions,
    store = transactionsStore,
    onAddTransaction,
    onUpdateTransaction,
    onDeleteTransaction,
  }: Props = $props();

  const transactions = $derived(explicitTransactions ?? store.transactions);

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

  function handleDraftChange(id: TransactionId, updates: Partial<Transaction>) {
    store.setRowDraftFields(id, updates);
  }

  function handleSaveRowDraft(id: TransactionId) {
    if (onUpdateTransaction) {
      const draft = store.rowDrafts[id];
      if (draft) void onUpdateTransaction(id, draft);
    } else {
      void store.saveRowDraft(id);
    }
  }

  function handleDeleteTransaction(id: TransactionId) {
    if (onDeleteTransaction) {
      onDeleteTransaction(id);
    } else {
      void store.delete(id);
    }
  }

  function handleAddTransaction(newTx: Omit<Transaction, "id">) {
    if (onAddTransaction) {
      onAddTransaction(newTx);
    } else {
      void store.create(newTx);
    }
  }

  function handleDiscardRowDraft(id: TransactionId) {
    store.discardRowDraft(id);
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

  const paginatedTransactions = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    return filteredTransactions.slice(start, start + pageSize);
  });

  let previousFilterSignature = $state("");
  $effect(() => {
    const signature = `${searchQuery}|${datePreset}|${customDateFrom}|${customDateTo}|${amountPointRange.join(",")}|${selectedTypeIds.join(",")}|${selectedMemberIds.join(",")}|${selectedAccountIds.join(",")}|${selectedCategoryIds.join(",")}|${selectedSubcategoryIds.join(",")}`;
    if (previousFilterSignature !== "" && previousFilterSignature !== signature) {
      currentPage = 1;
    }
    previousFilterSignature = signature;
  });

  const timelineGroups = $derived.by<readonly TimelineGroup[]>(() => {
    const referenceToday = "2026-10-05";
    const refTime = new Date(`${referenceToday}T00:00:00Z`).getTime();

    const oneDayAgoStr = new Date(refTime - 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
    const sevenDaysAgoStr = new Date(refTime - 7 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10);
    const refMonthStr = referenceToday.slice(0, 7);

    const groupsMap = new SvelteMap<string, { order: number; items: Transaction[] }>();

    for (const tx of paginatedTransactions) {
      let groupKey: string;
      let order: number;

      if (tx.date === referenceToday) {
        groupKey = "Today";
        order = 1;
      } else if (tx.date >= sevenDaysAgoStr && tx.date <= oneDayAgoStr) {
        groupKey = "Past 7 Days";
        order = 2;
      } else if (tx.date.slice(0, 7) === refMonthStr) {
        groupKey = "Earlier This Month";
        order = 3;
      } else {
        const [yearStr, monthStr] = tx.date.split("-");
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
        net += item.amount;
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
      rowDrafts={store.rowDrafts}
      hasRowDraft={(id) => store.hasRowDraft(id)}
      onSaveRowDraft={handleSaveRowDraft}
      onDiscardRowDraft={handleDiscardRowDraft}
      onDeleteTransaction={handleDeleteTransaction}
      onDraftChange={handleDraftChange}
    />

    <!-- Reusable Pagination Footer -->
    <PaginationFooter
      totalCount={totalFilteredCount}
      {pageSize}
      bind:currentPage
    />
  </div>

  <!-- Monochromatic Add Transaction Dialog Modal -->
  <AddTransactionModal
    bind:open={showAddModal}
    types={store.types}
    members={store.members}
    accounts={store.accounts}
    currency={store.baseCurrency}
    onAddTransaction={handleAddTransaction}
  />
{/if}
