<script lang="ts">
  import { TagIcon, ChevronDownIcon, ChevronRightIcon, CheckIcon } from "@lucide/svelte";
  import { SvelteMap } from "svelte/reactivity";
  import * as Popover from "$lib/components/ui/popover";
  import type { TransactionTypeItem, CategoryItem } from "$lib/features/categories/types";
  import type { TypeId, CategoryId, SubcategoryId } from "$lib/types";

  interface Props {
    types?: readonly TransactionTypeItem[];
    selectedTypeIds?: TypeId[];
    selectedCategoryIds?: CategoryId[];
    selectedSubcategoryIds?: SubcategoryId[];
  }

  let {
    types = [],
    selectedTypeIds = $bindable<TypeId[]>([]),
    selectedCategoryIds = $bindable<CategoryId[]>([]),
    selectedSubcategoryIds = $bindable<SubcategoryId[]>([]),
  }: Props = $props();

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

  const isCategoryFiltered = $derived(
    selectedTypeIds.length > 0 ||
      selectedCategoryIds.length > 0 ||
      selectedSubcategoryIds.length > 0,
  );

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

  function clearCategoryFilters() {
    selectedTypeIds = [];
    selectedCategoryIds = [];
    selectedSubcategoryIds = [];
  }
</script>

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
            onclick={clearCategoryFilters}
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
            <div class="text-muted-foreground/60 p-4 text-center text-xs">No categories found</div>
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
            <div class="text-muted-foreground/60 p-4 text-center text-xs">Select a category</div>
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
