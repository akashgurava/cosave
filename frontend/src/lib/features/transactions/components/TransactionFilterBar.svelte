<script lang="ts">
  import {
    SearchIcon,
    XIcon,
    CheckIcon,
    RotateCcwIcon,
    ChevronDownIcon,
    ChevronRightIcon,
    TagIcon,
    LandmarkIcon,
  } from "@lucide/svelte";
  import { SvelteMap } from "svelte/reactivity";
  import * as Popover from "$lib/components/ui/popover";
  import { Slider } from "$lib/components/ui/slider";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import type { DatePreset } from "../types";
  import type {
    TransactionTypeItem,
    CategoryItem,
    SubcategoryItem,
  } from "$lib/features/categories/types";
  import type { Member, Account } from "$lib/features/family/types";
  import {
    expectPresent,
    type MemberId,
    type AccountId,
    type CategoryId,
    type SubcategoryId,
    type TypeId,
  } from "$lib/types";

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

  // Flyout menu navigation state
  let menuMemberId = $state<MemberId | null>(null);
  let menuTypeId = $state<TypeId | number>(1 as TypeId);
  let menuCategoryId = $state<CategoryId | number | null>(null);

  $effect(() => {
    if (types.length > 0 && !types.some((t) => t.id === menuTypeId)) {
      const first = types[0];
      if (first) {
        menuTypeId = first.id;
        menuCategoryId = first.categories[0]?.id ?? null;
      }
    }
  });

  const memberMap = $derived(new SvelteMap(members.map((m) => [m.id, m])));
  const accountMap = $derived(new SvelteMap(accounts.map((a) => [a.id, a])));
  const typeMap = $derived(new SvelteMap(types.map((t) => [t.id, t])));
  const categoryMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of types) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, c);
      }
    }
    return map;
  });
  const subcategoryMap = $derived.by(() => {
    const map = new SvelteMap<SubcategoryId, SubcategoryItem>();
    for (const t of types) {
      for (const c of t.categories) {
        for (const s of c.subcategories) {
          map.set(s.id as SubcategoryId, s);
        }
      }
    }
    return map;
  });
  const typeByCategoryIdMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, TransactionTypeItem>();
    for (const t of types) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, t);
      }
    }
    return map;
  });

  function getMember(id: MemberId | number): Member {
    return expectPresent(
      memberMap.get(id as MemberId),
      "VIEW.FILTER_BAR.GET_MEMBER",
      `Member ${id} not found`,
    );
  }

  function getAccount(id: AccountId | number): Account {
    return expectPresent(
      accountMap.get(id as AccountId),
      "VIEW.FILTER_BAR.GET_ACCOUNT",
      `Account ${id} not found`,
    );
  }

  function getType(id: TypeId | number): TransactionTypeItem {
    return expectPresent(
      typeMap.get(id as TypeId),
      "VIEW.FILTER_BAR.GET_TYPE",
      `Type ${id} not found`,
    );
  }

  function getAccountName(acc: Account): string {
    return acc.type === "bank_account" ? acc.accountName : acc.cardName;
  }

  function getCategory(id: CategoryId | number): CategoryItem {
    return expectPresent(
      categoryMap.get(id as CategoryId),
      "VIEW.FILTER_BAR.GET_CATEGORY",
      `Category ${id} not found`,
    );
  }

  function getSubcategory(id: SubcategoryId | number): SubcategoryItem {
    return expectPresent(
      subcategoryMap.get(id as SubcategoryId),
      "VIEW.FILTER_BAR.GET_SUBCATEGORY",
      `Subcategory ${id} not found`,
    );
  }

  function getTypeForCategory(catId: CategoryId | number): TransactionTypeItem {
    return expectPresent(
      typeByCategoryIdMap.get(catId as CategoryId),
      "VIEW.FILTER_BAR.GET_TYPE_FOR_CATEGORY",
      `Type for category ${catId} not found`,
    );
  }

  const scopedMenuAccounts = $derived.by(() => {
    return menuMemberId ? accounts.filter((a) => a.ownerMemberId === menuMemberId) : accounts;
  });

  const scopedMenuCategories = $derived.by(() => {
    const t = typeMap.get(menuTypeId as TypeId);
    return t?.categories ?? [];
  });

  const activeMenuCategory = $derived.by(() => {
    return menuCategoryId ? (categoryMap.get(menuCategoryId as CategoryId) ?? null) : null;
  });

  const scopedMenuSubcategories = $derived.by(() => {
    return activeMenuCategory?.subcategories ?? [];
  });

  const isNoSubcategoryChecked = $derived.by(() => {
    return selectedSubcategoryIds.includes(0 as SubcategoryId);
  });

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

  function toggleMember(id: MemberId) {
    if (selectedMemberIds.includes(id)) {
      selectedMemberIds = selectedMemberIds.filter((m) => m !== id);
    } else {
      selectedMemberIds = [...selectedMemberIds, id];
    }
  }

  function memberOnly(id: MemberId) {
    selectedMemberIds = [id];
    selectedAccountIds = [];
  }

  function toggleAccount(id: AccountId) {
    if (selectedAccountIds.includes(id)) {
      selectedAccountIds = selectedAccountIds.filter((a) => a !== id);
    } else {
      selectedAccountIds = [...selectedAccountIds, id];
    }
  }

  function accountOnly(id: AccountId) {
    selectedAccountIds = [id];
  }

  function toggleType(id: TypeId) {
    if (selectedTypeIds.includes(id)) {
      selectedTypeIds = selectedTypeIds.filter((t) => t !== id);
    } else {
      selectedTypeIds = [...selectedTypeIds, id];
    }
  }

  function typeOnly(id: TypeId) {
    selectedTypeIds = [id];
    selectedCategoryIds = [];
    selectedSubcategoryIds = [];
  }

  function toggleCategory(id: CategoryId) {
    if (selectedCategoryIds.includes(id)) {
      selectedCategoryIds = selectedCategoryIds.filter((c) => c !== id);
    } else {
      selectedCategoryIds = [...selectedCategoryIds, id];
    }
  }

  function categoryOnly(id: CategoryId) {
    selectedCategoryIds = [id];
    selectedSubcategoryIds = [];
  }

  function toggleSubcategory(id: SubcategoryId) {
    if (selectedSubcategoryIds.includes(id)) {
      selectedSubcategoryIds = selectedSubcategoryIds.filter((s) => s !== id);
    } else {
      selectedSubcategoryIds = [...selectedSubcategoryIds, id];
    }
  }

  function subcategoryOnly(id: SubcategoryId) {
    selectedSubcategoryIds = [id];
  }

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
    <!-- 1. Account Flyout Filter -->
    <Popover.Root>
      <Popover.Trigger
        class="inline-flex h-8.5 items-center gap-2 rounded-lg border px-3 text-xs font-medium whitespace-nowrap transition-colors {isMemberAccountFiltered
          ? 'border-foreground bg-foreground text-background font-semibold shadow-xs'
          : 'border-border/40 text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <LandmarkIcon class="size-3.5" />
        <span>Account</span>
        <ChevronDownIcon class="size-3 opacity-60" />
      </Popover.Trigger>
      <Popover.Content
        side="bottom"
        align="start"
        sideOffset={6}
        class="border-border/60 bg-popover text-popover-foreground w-auto overflow-hidden rounded-xl border p-0 shadow-xl"
      >
        <div class="divide-border/40 flex h-72 divide-x">
          <!-- Left Layer: Members -->
          <div class="flex h-full w-40 shrink-0 flex-col">
            <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
              <!-- All Option -->
              <button
                type="button"
                onclick={() => {
                  menuMemberId = null;
                }}
                class="flex w-full items-center justify-between rounded-lg p-2 text-sm transition-colors {menuMemberId ===
                null
                  ? 'bg-muted text-foreground font-medium'
                  : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
              >
                <span>All</span>
                <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
              </button>

              <!-- Individual Members -->
              {#each members as m (m.id)}
                {@const isChecked = selectedMemberIds.includes(m.id)}
                {@const isHovered = menuMemberId === m.id}
                <div
                  role="button"
                  tabindex="0"
                  onmouseenter={() => (menuMemberId = m.id)}
                  onclick={() => (menuMemberId = m.id)}
                  onkeydown={(e) => {
                    if (e.key === "Enter") menuMemberId = m.id;
                  }}
                  class="group flex w-full cursor-pointer items-center justify-between rounded-lg p-2 text-sm transition-colors {isHovered
                    ? 'bg-muted text-foreground'
                    : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
                >
                  <button
                    type="button"
                    onclick={(e) => {
                      e.stopPropagation();
                      toggleMember(m.id);
                    }}
                    class="flex flex-1 items-center gap-2.5 text-left"
                  >
                    <span
                      class="flex size-4 items-center justify-center rounded border {isChecked
                        ? 'border-foreground bg-foreground text-background'
                        : 'border-muted-foreground/40 bg-transparent'}"
                    >
                      {#if isChecked}
                        <CheckIcon class="size-3 stroke-3" />
                      {/if}
                    </span>
                    <span class={isChecked ? "text-foreground font-medium" : ""}
                      >{m.memberName}</span
                    >
                  </button>

                  <div class="flex items-center gap-1.5">
                    <button
                      type="button"
                      onclick={(e) => {
                        e.stopPropagation();
                        memberOnly(m.id);
                      }}
                      class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[10px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                      title="Show only {m.memberName}"
                    >
                      Only
                    </button>
                    <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
                  </div>
                </div>
              {/each}
            </div>

            <!-- Footer Quick Filter -->
            <div class="border-border/40 border-t p-2">
              <button
                type="button"
                onclick={() => {
                  selectedMemberIds = [];
                  selectedAccountIds = [];
                }}
                class="hover:bg-muted text-muted-foreground hover:text-foreground w-full rounded-md py-1.5 text-center text-xs transition-colors"
              >
                Clear Account Filters
              </button>
            </div>
          </div>

          <!-- Right Layer: Accounts for Selected Member (or All Accounts) -->
          <div class="flex h-full w-56 flex-col">
            <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
              {#if scopedMenuAccounts.length === 0}
                <div class="text-muted-foreground/60 p-4 text-center text-xs">
                  No accounts found
                </div>
              {:else}
                {#each scopedMenuAccounts as acc (acc.id)}
                  {@const isChecked = selectedAccountIds.includes(acc.id)}
                  <div
                    class="hover:bg-muted/50 group flex items-center justify-between rounded-lg p-2 text-sm transition-colors"
                  >
                    <button
                      type="button"
                      onclick={() => toggleAccount(acc.id)}
                      class="flex flex-1 items-center gap-2.5 text-left"
                    >
                      <span
                        class="flex size-4 items-center justify-center rounded border {isChecked
                          ? 'border-foreground bg-foreground text-background'
                          : 'border-muted-foreground/40 bg-transparent'}"
                      >
                        {#if isChecked}
                          <CheckIcon class="size-3 stroke-3" />
                        {/if}
                      </span>
                      <div class="truncate">
                        <div class="truncate font-medium {isChecked ? 'text-foreground' : ''}">
                          {getAccountName(acc)}
                        </div>
                        <div class="text-muted-foreground font-mono text-[10px]">
                          {acc.bankName} ····{acc.last4}
                        </div>
                      </div>
                    </button>

                    <button
                      type="button"
                      onclick={() => accountOnly(acc.id)}
                      class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[10px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                      title="Show only {getAccountName(acc)}"
                    >
                      Only
                    </button>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </Popover.Content>
    </Popover.Root>

    <!-- 2. Category Flyout Filter (3-Level Deep Cascading Filter) -->
    <Popover.Root>
      <Popover.Trigger
        class="inline-flex h-8.5 items-center gap-2 rounded-lg border px-3 text-xs font-medium whitespace-nowrap transition-colors {isCategoryFiltered
          ? 'border-foreground bg-foreground text-background font-semibold shadow-xs'
          : 'border-border/40 text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <TagIcon class="size-3.5" />
        <span>Category</span>
        <ChevronDownIcon class="size-3 opacity-60" />
      </Popover.Trigger>
      <Popover.Content
        side="bottom"
        align="start"
        sideOffset={6}
        class="border-border/60 bg-popover text-popover-foreground w-auto overflow-hidden rounded-xl border p-0 shadow-xl"
      >
        <div class="divide-border/40 flex h-72 divide-x">
          <!-- Layer 1: Flow Type -->
          <div class="flex h-full w-44 shrink-0 flex-col">
            <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
              {#each types as t (t.id)}
                {@const isChecked = selectedTypeIds.includes(t.id as TypeId)}
                {@const isHovered = menuTypeId === t.id}
                <div
                  role="button"
                  tabindex="0"
                  onmouseenter={() => {
                    menuTypeId = t.id;
                    const firstCat = t.categories[0];
                    menuCategoryId = firstCat ? firstCat.id : null;
                  }}
                  onclick={() => {
                    menuTypeId = t.id;
                    const firstCat = t.categories[0];
                    menuCategoryId = firstCat ? firstCat.id : null;
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      menuTypeId = t.id;
                      const firstCat = t.categories[0];
                      menuCategoryId = firstCat ? firstCat.id : null;
                    }
                  }}
                  class="group flex w-full cursor-pointer items-center justify-between rounded-lg p-1.5 text-xs transition-colors {isHovered
                    ? 'bg-muted text-foreground'
                    : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
                >
                  <button
                    type="button"
                    onclick={(e) => {
                      e.stopPropagation();
                      toggleType(t.id as TypeId);
                    }}
                    class="flex flex-1 items-center gap-2 truncate text-left"
                  >
                    <span
                      class="flex size-3.5 shrink-0 items-center justify-center rounded border {isChecked
                        ? 'border-foreground bg-foreground text-background'
                        : 'border-muted-foreground/40 bg-transparent'}"
                    >
                      {#if isChecked}
                        <CheckIcon class="size-2.5 stroke-3" />
                      {/if}
                    </span>
                    <span class="size-2 shrink-0 rounded-full" style="background-color: {t.color};"
                    ></span>
                    <span class="capitalize {isChecked ? 'text-foreground font-medium' : ''}">
                      {t.name}
                    </span>
                  </button>

                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      onclick={(e) => {
                        e.stopPropagation();
                        typeOnly(t.id as TypeId);
                      }}
                      class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[9px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                      title="Show only {t.name}"
                    >
                      Only
                    </button>
                    <ChevronRightIcon class="text-muted-foreground/60 size-3" />
                  </div>
                </div>
              {/each}
            </div>

            <!-- Footer Clear Filter -->
            <div class="border-border/40 border-t p-2">
              <button
                type="button"
                onclick={() => {
                  selectedTypeIds = [];
                  selectedCategoryIds = [];
                  selectedSubcategoryIds = [];
                }}
                class="hover:bg-muted text-muted-foreground hover:text-foreground w-full rounded-md py-1.5 text-center text-xs transition-colors"
              >
                Clear Category Filters
              </button>
            </div>
          </div>

          <!-- Layer 2: Categories for Selected Flow Type -->
          <div class="flex h-full w-48 shrink-0 flex-col">
            <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
              {#if scopedMenuCategories.length === 0}
                <div class="text-muted-foreground/60 p-4 text-center text-xs">
                  No categories found
                </div>
              {:else}
                {#each scopedMenuCategories as cat (cat.id)}
                  {@const isChecked = selectedCategoryIds.includes(cat.id as CategoryId)}
                  {@const isHovered = menuCategoryId === cat.id}
                  <div
                    role="button"
                    tabindex="0"
                    onmouseenter={() => (menuCategoryId = cat.id)}
                    onclick={() => (menuCategoryId = cat.id)}
                    onkeydown={(e) => {
                      if (e.key === "Enter") menuCategoryId = cat.id;
                    }}
                    class="group flex w-full cursor-pointer items-center justify-between rounded-lg p-1.5 text-xs transition-colors {isHovered
                      ? 'bg-muted text-foreground'
                      : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
                  >
                    <button
                      type="button"
                      onclick={(e) => {
                        e.stopPropagation();
                        toggleCategory(cat.id as CategoryId);
                      }}
                      class="flex flex-1 items-center gap-2 truncate text-left"
                    >
                      <span
                        class="flex size-3.5 shrink-0 items-center justify-center rounded border {isChecked
                          ? 'border-foreground bg-foreground text-background'
                          : 'border-muted-foreground/40 bg-transparent'}"
                      >
                        {#if isChecked}
                          <CheckIcon class="size-2.5 stroke-3" />
                        {/if}
                      </span>
                      <span class="truncate {isChecked ? 'text-foreground font-medium' : ''}">
                        {cat.name}
                      </span>
                    </button>

                    <div class="flex items-center gap-1">
                      <button
                        type="button"
                        onclick={(e) => {
                          e.stopPropagation();
                          categoryOnly(cat.id as CategoryId);
                        }}
                        class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[9px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                        title="Show only {cat.name}"
                      >
                        Only
                      </button>
                      <ChevronRightIcon class="text-muted-foreground/60 size-3" />
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>

          <!-- Layer 3: Subcategories for Selected Category -->
          <div class="flex h-full w-52 flex-col">
            <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
              {#if !activeMenuCategory}
                <div class="text-muted-foreground/60 p-4 text-center text-xs">
                  Select a category
                </div>
              {:else}
                <!-- Option for (No Subcategory / Unassigned) -->
                <div
                  class="hover:bg-muted/50 group flex items-center justify-between rounded-lg p-1.5 text-xs transition-colors"
                >
                  <button
                    type="button"
                    onclick={() => toggleSubcategory(0 as SubcategoryId)}
                    class="flex flex-1 items-center gap-2 text-left"
                  >
                    <span
                      class="flex size-3.5 shrink-0 items-center justify-center rounded border {isNoSubcategoryChecked
                        ? 'border-foreground bg-foreground text-background'
                        : 'border-muted-foreground/40 bg-transparent'}"
                    >
                      {#if isNoSubcategoryChecked}
                        <CheckIcon class="size-2.5 stroke-3" />
                      {/if}
                    </span>
                    <span
                      class="italic {isNoSubcategoryChecked
                        ? 'text-foreground font-medium'
                        : 'text-muted-foreground'}"
                    >
                      (No Subcategory)
                    </span>
                  </button>

                  <button
                    type="button"
                    onclick={() => subcategoryOnly(0 as SubcategoryId)}
                    class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[9px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                    title="Show only unassigned"
                  >
                    Only
                  </button>
                </div>

                <!-- Regular Subcategories -->
                {#each scopedMenuSubcategories as sub (sub.id)}
                  {@const isChecked = selectedSubcategoryIds.includes(sub.id as SubcategoryId)}
                  <div
                    class="hover:bg-muted/50 group flex items-center justify-between rounded-lg p-1.5 text-xs transition-colors"
                  >
                    <button
                      type="button"
                      onclick={() => toggleSubcategory(sub.id as SubcategoryId)}
                      class="flex flex-1 items-center gap-2 truncate text-left"
                    >
                      <span
                        class="flex size-3.5 shrink-0 items-center justify-center rounded border {isChecked
                          ? 'border-foreground bg-foreground text-background'
                          : 'border-muted-foreground/40 bg-transparent'}"
                      >
                        {#if isChecked}
                          <CheckIcon class="size-2.5 stroke-3" />
                        {/if}
                      </span>
                      <span class="truncate {isChecked ? 'text-foreground font-medium' : ''}">
                        {sub.name}
                      </span>
                    </button>

                    <button
                      type="button"
                      onclick={() => subcategoryOnly(sub.id as SubcategoryId)}
                      class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[9px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                      title="Show only {sub.name}"
                    >
                      Only
                    </button>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </Popover.Content>
    </Popover.Root>
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
    <div class="flex flex-wrap items-center gap-1.5">
      {#each selectedMemberIds as id (id)}
        {@const m = getMember(id)}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span>{m.memberName}</span>
          <button type="button" onclick={() => toggleMember(id)}><XIcon class="size-2.5" /></button>
        </Badge>
      {/each}

      {#each selectedAccountIds as id (id)}
        {@const acc = getAccount(id)}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span>{getAccountName(acc)}</span>
          <button type="button" onclick={() => toggleAccount(id)}><XIcon class="size-2.5" /></button
          >
        </Badge>
      {/each}

      {#each selectedTypeIds as id (id)}
        {@const t = getType(id)}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px] capitalize">
          <span class="size-1.5 rounded-full" style="background-color: {t.color};"></span>
          <span>{t.name}</span>
          <button type="button" onclick={() => toggleType(id)}><XIcon class="size-2.5" /></button>
        </Badge>
      {/each}

      {#each selectedCategoryIds as id (id)}
        {@const cat = getCategory(id)}
        {@const catType = getTypeForCategory(id)}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span class="size-1.5 rounded-full" style="background-color: {catType.color};"></span>
          <span>{cat.name}</span>
          <button type="button" onclick={() => toggleCategory(id)}
            ><XIcon class="size-2.5" /></button
          >
        </Badge>
      {/each}

      {#each selectedSubcategoryIds as id (id)}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span>{Number(id) > 0 ? getSubcategory(id).name : "No Subcategory"}</span>
          <button type="button" onclick={() => toggleSubcategory(id)}
            ><XIcon class="size-2.5" /></button
          >
        </Badge>
      {/each}

      {#if datePreset !== "all"}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span>Range: {datePreset}</span>
          <button type="button" onclick={() => (datePreset = "all")}
            ><XIcon class="size-2.5" /></button
          >
        </Badge>
      {/if}

      {#if isAmountFiltered}
        <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
          <span>Amount: {amountDisplayLabel}</span>
          <button type="button" onclick={() => (amountPointRange = [0, 5])}
            ><XIcon class="size-2.5" /></button
          >
        </Badge>
      {/if}

      {#if hasActiveFilters}
        <button
          type="button"
          onclick={onResetAll}
          class="text-muted-foreground hover:text-foreground ml-1 inline-flex items-center gap-1 font-mono text-[11px] underline"
        >
          <RotateCcwIcon class="size-3" />
          Reset All
        </button>
      {/if}
    </div>
  </div>
</div>
