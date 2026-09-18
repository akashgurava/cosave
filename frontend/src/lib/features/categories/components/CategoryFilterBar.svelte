<script lang="ts">
  import { categoryStore } from "../store";
  import LayersIcon from "@lucide/svelte/icons/layers";

  interface Props {
    selectedTypes: string[];
    onToggleType: (typeName: string) => void;
    onSelectOnly: (typeName: string) => void;
    onSelectAll: () => void;
  }

  let { selectedTypes, onToggleType, onSelectOnly, onSelectAll }: Props = $props();

  const isAllSelected = $derived(
    selectedTypes.length === 0 || selectedTypes.length === categoryStore.types.length,
  );

  function isTypeActive(typeName: string): boolean {
    if (isAllSelected) return true;
    return selectedTypes.some((t) => t.toLowerCase() === typeName.toLowerCase());
  }

  const displayedTypesCount = $derived(
    categoryStore.types.filter((t) => isTypeActive(t.name)).length,
  );
  const displayedCategories = $derived(
    categoryStore.categories.filter((c) => isTypeActive(c.type)),
  );
  const displayedCategoriesCount = $derived(displayedCategories.length);
  const displayedSubcategoriesCount = $derived(
    displayedCategories.reduce((acc, c) => acc + c.subcategories.length, 0),
  );
</script>

<div
  class="bg-card flex flex-wrap items-center justify-between gap-3 rounded-lg border p-3 shadow-xs"
>
  <!-- Filter Pills -->
  <div class="flex flex-wrap items-center gap-2">
    <span class="text-muted-foreground mr-1 flex items-center gap-1.5 text-sm font-medium">
      <LayersIcon class="size-4" /> Filter:
    </span>

    <button
      type="button"
      class={`inline-flex h-9 items-center gap-1.5 rounded-lg px-3.5 text-sm font-medium transition-all ${
        isAllSelected
          ? "bg-foreground text-background font-semibold shadow-xs"
          : "text-muted-foreground bg-muted/40 hover:bg-muted hover:text-foreground"
      }`}
      onclick={onSelectAll}
    >
      All Types
    </button>

    {#each categoryStore.types as t (t.id)}
      {@const colorObj = categoryStore.getTypeColor(t.name)}
      {@const active = isTypeActive(t.name)}
      <div
        class={`group inline-flex h-9 items-center rounded-lg border text-sm font-medium transition-all ${
          active
            ? "border-border bg-muted/60 text-foreground shadow-2xs"
            : "bg-muted/20 text-muted-foreground border-transparent opacity-50 hover:opacity-80"
        }`}
      >
        <button
          type="button"
          class="hover:text-foreground inline-flex items-center gap-2 px-3 py-1.5 transition-colors"
          onclick={() => onToggleType(t.name)}
          title={active ? `Hide ${t.name}` : `Show ${t.name}`}
        >
          <span
            class={`size-2.5 rounded-full transition-opacity ${active ? "opacity-100 shadow-xs" : "opacity-40"}`}
            style="background-color: {colorObj.solid};"
          ></span>
          <span class={active ? "font-semibold" : "font-normal"}>{t.name}</span>
        </button>
        <button
          type="button"
          class="text-muted-foreground hover:text-foreground hover:bg-background bg-muted/50 mr-1.5 rounded-md px-2 py-0.5 text-[11px] font-semibold tracking-wide uppercase opacity-80 transition-all sm:opacity-40 sm:group-hover:opacity-100"
          onclick={(e) => {
            e.stopPropagation();
            onSelectOnly(t.name);
          }}
          title={`Show only ${t.name}`}
        >
          only
        </button>
      </div>
    {/each}
  </div>

  <!-- Dynamic Metrics Overview -->
  <div class="text-muted-foreground flex items-center gap-3 text-xs sm:text-sm">
    <span
      ><strong>{displayedTypesCount}</strong> {displayedTypesCount === 1 ? "Type" : "Types"}</span
    >
    <span>&bull;</span>
    <span
      ><strong>{displayedCategoriesCount}</strong>
      {displayedCategoriesCount === 1 ? "Category" : "Categories"}</span
    >
    <span>&bull;</span>
    <span
      ><strong>{displayedSubcategoriesCount}</strong>
      {displayedSubcategoriesCount === 1 ? "Subcategory" : "Subcategories"}</span
    >
  </div>
</div>
