<script lang="ts">
  import { SearchIcon, XIcon } from "@lucide/svelte";
  import * as Popover from "$lib/components/ui/popover";
  import { Slider } from "$lib/components/ui/slider";
  import { Input } from "$lib/components/ui/input";
  import type { DatePreset } from "../types";
  import type { TransactionTypeItem } from "$lib/features/categories/types";
  import type { Member, Account } from "$lib/features/family/types";
  import type { MemberId, AccountId, CategoryId, SubcategoryId, TypeId } from "$lib/types";
  import AccountFilterPopover from "./filters/AccountFilterPopover.svelte";
  import CategoryFilterPopover from "./filters/CategoryFilterPopover.svelte";
  import FilterBadges from "./filters/FilterBadges.svelte";

  interface Props {
    types?: readonly TransactionTypeItem[];
    members?: readonly Member[];
    accounts?: readonly Account[];
    searchQuery?: string;
    datePreset?: DatePreset;
    customDateFrom?: string;
    customDateTo?: string;
    amountPointRange?: number[];
    selectedMemberIds?: MemberId[];
    selectedAccountIds?: AccountId[];
    selectedTypeIds?: TypeId[];
    selectedCategoryIds?: CategoryId[];
    selectedSubcategoryIds?: SubcategoryId[];
    onResetAll: () => void;
  }

  let {
    types = [],
    members = [],
    accounts = [],
    searchQuery = $bindable(""),
    datePreset = $bindable<DatePreset>("all"),
    customDateFrom = $bindable(""),
    customDateTo = $bindable(""),
    amountPointRange = $bindable<number[]>([0, 5]),
    selectedMemberIds = $bindable<MemberId[]>([]),
    selectedAccountIds = $bindable<AccountId[]>([]),
    selectedTypeIds = $bindable<TypeId[]>([]),
    selectedCategoryIds = $bindable<CategoryId[]>([]),
    selectedSubcategoryIds = $bindable<SubcategoryId[]>([]),
    onResetAll,
  }: Props = $props();

  // Fixed-point amount slider points: 0, 100, 500, 1000, 2000, > 2000
  const AMOUNT_POINT_LABELS = ["$0", "$100", "$500", "$1,000", "$2,000", "> $2,000"] as const;
  const AMOUNT_TICK_LABELS = ["$0", "$100", "$500", "$1k", "$2k", "> $2k"] as const;

  const lowPointIdx = $derived(amountPointRange[0] ?? 0);
  const highPointIdx = $derived(amountPointRange[1] ?? 5);
  const isAmountFiltered = $derived(lowPointIdx > 0 || highPointIdx < 5);

  const amountDisplayLabel = $derived.by<string>(() => {
    if (lowPointIdx === 0 && highPointIdx === 5) {
      return "$0 – > $2,000";
    }
    if (lowPointIdx === 0) {
      return `< ${AMOUNT_POINT_LABELS[highPointIdx]}`;
    }
    if (highPointIdx === 5) {
      if (lowPointIdx === 5) return "> $2,000";
      return `≥ ${AMOUNT_POINT_LABELS[lowPointIdx]}`;
    }
    return `${AMOUNT_POINT_LABELS[lowPointIdx]} – ${AMOUNT_POINT_LABELS[highPointIdx]}`;
  });

  const isMemberAccountFiltered = $derived(
    selectedMemberIds.length > 0 || selectedAccountIds.length > 0,
  );

  const isCategoryFiltered = $derived(
    selectedTypeIds.length > 0 ||
      selectedCategoryIds.length > 0 ||
      selectedSubcategoryIds.length > 0,
  );

  const hasActiveFilters = $derived(
    searchQuery.trim().length > 0 ||
      datePreset !== "all" ||
      Boolean(customDateFrom) ||
      Boolean(customDateTo) ||
      isAmountFiltered ||
      isMemberAccountFiltered ||
      isCategoryFiltered,
  );

  function toggleDatePreset(preset: DatePreset) {
    if (datePreset === preset && !customDateFrom && !customDateTo) {
      datePreset = "all";
    } else {
      datePreset = preset;
      customDateFrom = "";
      customDateTo = "";
    }
  }
</script>

<div
  class="border-border/40 bg-card space-y-2.5 rounded-xl border p-3 shadow-xs sm:space-y-3.5 sm:p-3.5"
>
  <!-- Level 1: Hierarchical Cascading Flyouts -->
  <div class="flex flex-wrap items-center gap-2 sm:gap-2.5">
    <AccountFilterPopover {members} {accounts} bind:selectedMemberIds bind:selectedAccountIds />

    <CategoryFilterPopover
      {types}
      bind:selectedTypeIds
      bind:selectedCategoryIds
      bind:selectedSubcategoryIds
    />
  </div>

  <!-- Level 2: Presets & Quick Filters -->
  <div class="flex flex-col gap-2 pt-0.5 sm:flex-row sm:items-center sm:justify-between sm:gap-3">
    <!-- Date Presets: strictly 1 horizontally scrollable row on mobile -->
    <div
      class="flex max-w-full scrollbar-none items-center gap-1.5 overflow-x-auto pb-0.5 [&::-webkit-scrollbar]:hidden"
    >
      <span class="text-muted-foreground shrink-0 text-xs font-semibold">Date:</span>
      {#each ["all", "1d", "3d", "7d", "1m", "3m", "6m", "1y"] as const as preset (preset)}
        <button
          type="button"
          onclick={() => toggleDatePreset(preset)}
          class="h-7 shrink-0 rounded-md px-2.5 font-mono text-xs transition-colors {datePreset ===
            preset &&
          !customDateFrom &&
          !customDateTo
            ? 'bg-foreground text-background font-semibold shadow-xs'
            : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
        >
          {preset === "all" ? "All" : preset.toUpperCase()}
        </button>
      {/each}

      <!-- Custom Date Range Popover -->
      <Popover.Root>
        <Popover.Trigger
          class="h-7 shrink-0 rounded-md px-2.5 font-mono text-xs transition-colors {customDateFrom ||
          customDateTo
            ? 'bg-foreground text-background font-semibold shadow-xs'
            : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
        >
          {customDateFrom || customDateTo ? `${customDateFrom} → ${customDateTo}` : "Custom"}
        </Popover.Trigger>
        <Popover.Content align="start" side="bottom" sideOffset={6} class="w-72 space-y-3 p-3">
          <div class="space-y-1">
            <span class="text-muted-foreground font-mono text-[10px] uppercase">From Date</span>
            <input
              type="date"
              bind:value={customDateFrom}
              onchange={() => {
                datePreset = "custom";
              }}
              class="border-border/40 bg-background w-full rounded-md border px-2 py-1 font-mono text-xs"
            />
          </div>
          <div class="space-y-1">
            <span class="text-muted-foreground font-mono text-[10px] uppercase">To Date</span>
            <input
              type="date"
              bind:value={customDateTo}
              onchange={() => {
                datePreset = "custom";
              }}
              class="border-border/40 bg-background w-full rounded-md border px-2 py-1 font-mono text-xs"
            />
          </div>
          <div class="flex justify-end gap-2 pt-1">
            <button
              type="button"
              onclick={() => {
                customDateFrom = "";
                customDateTo = "";
                datePreset = "all";
              }}
              class="hover:bg-muted text-muted-foreground rounded px-2 py-1 text-xs"
            >
              Reset
            </button>
          </div>
        </Popover.Content>
      </Popover.Root>
    </div>

    <!-- Amount Range Fixed Slider -->
    <div class="flex items-center gap-2">
      <span class="text-muted-foreground text-xs font-semibold">Amount:</span>
      <div
        class="border-border/40 bg-background/50 flex items-center rounded-lg border px-3 py-1.5"
      >
        <div class="w-44 space-y-1 sm:w-52">
          <Slider type="multiple" bind:value={amountPointRange} min={0} max={5} step={1} />
          <div
            class="text-muted-foreground flex justify-between px-0.5 font-mono text-[9px] select-none"
          >
            {#each AMOUNT_TICK_LABELS as label (label)}
              <span>{label}</span>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Level 3: Search Bar with Active Filter Badges & Reset -->
  <div class="border-border/30 flex flex-wrap items-center gap-2 border-t pt-2 sm:pt-2.5">
    <div class="relative min-w-56 flex-1">
      <SearchIcon
        class="text-muted-foreground absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2"
      />
      <Input
        type="text"
        placeholder="Search transactions, descriptions, payees..."
        bind:value={searchQuery}
        class="border-border/40 bg-background/50 h-8 pr-7 pl-8 text-xs"
      />
      {#if searchQuery}
        <button
          type="button"
          onclick={() => (searchQuery = "")}
          class="text-muted-foreground hover:text-foreground absolute top-1/2 right-2 -translate-y-1/2"
        >
          <XIcon class="size-3" />
        </button>
      {/if}
    </div>

    <!-- Active Filter Badges -->
    <FilterBadges
      {members}
      {accounts}
      {types}
      bind:selectedMemberIds
      bind:selectedAccountIds
      bind:selectedTypeIds
      bind:selectedCategoryIds
      bind:selectedSubcategoryIds
      bind:datePreset
      bind:amountPointRange
      {amountDisplayLabel}
      {isAmountFiltered}
      {hasActiveFilters}
      {onResetAll}
    />
  </div>
</div>
