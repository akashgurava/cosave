<script lang="ts">
  import { XIcon, RotateCcwIcon } from "@lucide/svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { Badge } from "$lib/components/ui/badge";
  import type { DatePreset } from "../../types";
  import type {
    TransactionTypeItem,
    CategoryItem,
    SubcategoryItem,
  } from "$lib/features/categories/types";
  import type { Member, Account } from "$lib/features/family/types";
  import {
    expectPresent,
    type AccountId,
    type CategoryId,
    type MemberId,
    type SubcategoryId,
    type TypeId,
  } from "$lib/types";

  interface Props {
    members?: readonly Member[];
    accounts?: readonly Account[];
    types?: readonly TransactionTypeItem[];
    selectedMemberIds?: MemberId[];
    selectedAccountIds?: AccountId[];
    selectedTypeIds?: TypeId[];
    selectedCategoryIds?: CategoryId[];
    selectedSubcategoryIds?: SubcategoryId[];
    datePreset?: DatePreset;
    amountDisplayLabel: string;
    isAmountFiltered: boolean;
    hasActiveFilters: boolean;
    onResetAmount: () => void;
    onResetAll: () => void;
  }

  let {
    members = [],
    accounts = [],
    types = [],
    selectedMemberIds = $bindable<MemberId[]>([]),
    selectedAccountIds = $bindable<AccountId[]>([]),
    selectedTypeIds = $bindable<TypeId[]>([]),
    selectedCategoryIds = $bindable<CategoryId[]>([]),
    selectedSubcategoryIds = $bindable<SubcategoryId[]>([]),
    datePreset = $bindable<DatePreset>("all"),
    amountDisplayLabel,
    isAmountFiltered,
    hasActiveFilters,
    onResetAmount,
    onResetAll,
  }: Props = $props();

  const memberMap = $derived(new SvelteMap(members.map((m) => [m.id, m])));
  const accountMap = $derived(new SvelteMap(accounts.map((a) => [a.id, a])));
  const typeMap = $derived(new SvelteMap(types.map((t) => [t.id, t])));

  const categoryMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of types) {
      for (const c of t.categories) {
        map.set(c.id, c);
      }
    }
    return map;
  });

  const subcategoryMap = $derived.by(() => {
    const map = new SvelteMap<SubcategoryId, SubcategoryItem>();
    for (const t of types) {
      for (const c of t.categories) {
        for (const s of c.subcategories) {
          map.set(s.id, s);
        }
      }
    }
    return map;
  });

  const typeByCategoryIdMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, TransactionTypeItem>();
    for (const t of types) {
      for (const c of t.categories) {
        map.set(c.id, t);
      }
    }
    return map;
  });

  function getMember(id: MemberId): Member {
    return expectPresent(
      memberMap.get(id),
      "VIEW.FILTER_BADGES.GET_MEMBER",
      `Member ${id} not found`,
    );
  }

  function getAccount(id: AccountId): Account {
    return expectPresent(
      accountMap.get(id),
      "VIEW.FILTER_BADGES.GET_ACCOUNT",
      `Account ${id} not found`,
    );
  }

  function getType(id: TypeId): TransactionTypeItem {
    return expectPresent(typeMap.get(id), "VIEW.FILTER_BADGES.GET_TYPE", `Type ${id} not found`);
  }

  function getCategory(id: CategoryId): CategoryItem {
    return expectPresent(
      categoryMap.get(id),
      "VIEW.FILTER_BADGES.GET_CATEGORY",
      `Category ${id} not found`,
    );
  }

  function getSubcategory(id: SubcategoryId): SubcategoryItem {
    return expectPresent(
      subcategoryMap.get(id),
      "VIEW.FILTER_BADGES.GET_SUBCATEGORY",
      `Subcategory ${id} not found`,
    );
  }

  function getTypeForCategory(catId: CategoryId): TransactionTypeItem {
    return expectPresent(
      typeByCategoryIdMap.get(catId),
      "VIEW.FILTER_BADGES.GET_TYPE_FOR_CATEGORY",
      `Type for category ${catId} not found`,
    );
  }

  function getAccountName(acc: Account): string {
    return acc.type === "bank_account" ? acc.accountName : acc.cardName;
  }

  function removeMember(id: MemberId) {
    selectedMemberIds = selectedMemberIds.filter((m) => m !== id);
  }

  function removeAccount(id: AccountId) {
    selectedAccountIds = selectedAccountIds.filter((a) => a !== id);
  }

  function removeType(id: TypeId) {
    selectedTypeIds = selectedTypeIds.filter((t) => t !== id);
  }

  function removeCategory(id: CategoryId) {
    selectedCategoryIds = selectedCategoryIds.filter((c) => c !== id);
  }

  function removeSubcategory(id: SubcategoryId) {
    selectedSubcategoryIds = selectedSubcategoryIds.filter((s) => s !== id);
  }
</script>

<div class="flex flex-wrap items-center gap-1.5">
  {#each selectedMemberIds as id (id)}
    {@const m = getMember(id)}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span>{m.memberName}</span>
      <button type="button" onclick={() => removeMember(id)}><XIcon class="size-2.5" /></button>
    </Badge>
  {/each}

  {#each selectedAccountIds as id (id)}
    {@const acc = getAccount(id)}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span>{getAccountName(acc)}</span>
      <button type="button" onclick={() => removeAccount(id)}><XIcon class="size-2.5" /></button>
    </Badge>
  {/each}

  {#each selectedTypeIds as id (id)}
    {@const t = getType(id)}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px] capitalize">
      <span class="size-1.5 rounded-full" style="background-color: {t.color};"></span>
      <span>{t.name}</span>
      <button type="button" onclick={() => removeType(id)}><XIcon class="size-2.5" /></button>
    </Badge>
  {/each}

  {#each selectedCategoryIds as id (id)}
    {@const cat = getCategory(id)}
    {@const catType = getTypeForCategory(id)}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span class="size-1.5 rounded-full" style="background-color: {catType.color};"></span>
      <span>{cat.name}</span>
      <button type="button" onclick={() => removeCategory(id)}><XIcon class="size-2.5" /></button>
    </Badge>
  {/each}

  {#each selectedSubcategoryIds as id (id)}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span>{Number(id) > 0 ? getSubcategory(id).name : "No Subcategory"}</span>
      <button type="button" onclick={() => removeSubcategory(id)}><XIcon class="size-2.5" /></button
      >
    </Badge>
  {/each}

  {#if datePreset !== "all"}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span>Range: {datePreset}</span>
      <button type="button" onclick={() => (datePreset = "all")}><XIcon class="size-2.5" /></button>
    </Badge>
  {/if}

  {#if isAmountFiltered}
    <Badge variant="secondary" class="h-6 gap-1 px-2 font-mono text-[10px]">
      <span>Amount: {amountDisplayLabel}</span>
      <button type="button" onclick={onResetAmount}><XIcon class="size-2.5" /></button>
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
