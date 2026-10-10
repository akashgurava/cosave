<script lang="ts">
  import { CheckIcon } from "@lucide/svelte";
  import * as Popover from "$lib/components/ui/popover";
  import type { Transaction } from "../../types";
  import type {
    CategoryItem,
    SubcategoryItem,
    TransactionTypeItem,
  } from "$lib/features/categories/types";
  import type { TransactionId, CategoryId, SubcategoryId, TypeId } from "$lib/types";

  interface Props {
    txId: TransactionId;
    effectiveTx: Transaction;
    txType: TransactionTypeItem;
    effectiveTypeColor: string;
    category: CategoryItem;
    subcategory?: SubcategoryItem;
    types: readonly TransactionTypeItem[];
    onDraftChange: (id: TransactionId, updates: Partial<Transaction>) => void;
  }

  let {
    txId,
    effectiveTx,
    txType,
    effectiveTypeColor,
    category,
    subcategory,
    types,
    onDraftChange,
  }: Props = $props();

  let open = $state(false);
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class="text-foreground hover:bg-muted/60 flex w-full max-w-full items-center gap-1.5 truncate rounded px-1.5 py-0.5 text-left text-[11px] transition-colors"
    title="{effectiveTx.type} › {category.name}{subcategory ? ` › ${subcategory.name}` : ''}"
  >
    <span class="text-foreground shrink-0 font-medium capitalize">{effectiveTx.type}</span>
    <span class="text-muted-foreground/40 font-mono">›</span>
    <span class="text-foreground shrink-0 font-medium">{category.name}</span>
    {#if subcategory}
      <span class="text-muted-foreground/40 font-mono">›</span>
      <span class="text-muted-foreground shrink-0">{subcategory.name}</span>
    {/if}
  </Popover.Trigger>
  <Popover.Content align="start" side="bottom" sideOffset={4} class="w-72 space-y-1.5 p-1.5">
    <!-- Dynamic flow type selector buttons -->
    <div class="border-border/40 flex flex-wrap items-center gap-1 border-b pb-1.5">
      {#each types as t (t.id)}
        <button
          type="button"
          onclick={() => {
            const defaultCat = t.categories[0];
            onDraftChange(txId, {
              typeId: t.id as TypeId,
              type: t.name,
              typeColor: t.color,
              categoryId: (defaultCat?.id ?? effectiveTx.categoryId) as CategoryId,
              subcategoryId: undefined,
            });
          }}
          class="flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors {effectiveTx.typeId ===
            t.id || effectiveTx.type.toLowerCase() === t.name.toLowerCase()
            ? 'bg-muted text-foreground font-semibold'
            : 'text-muted-foreground hover:bg-muted/50'}"
        >
          <span class="size-1.5 rounded-full" style="background-color: {t.color};"></span>
          <span>{t.name}</span>
        </button>
      {/each}
    </div>

    <!-- Categories for selected type -->
    <div class="max-h-60 space-y-1 overflow-y-auto">
      {#each txType.categories as cat (cat.id)}
        <div class="space-y-0.5">
          <button
            type="button"
            onclick={() => {
              onDraftChange(txId, {
                categoryId: cat.id as CategoryId,
                typeId: txType.id as TypeId,
                type: txType.name,
                typeColor: txType.color,
                subcategoryId: undefined,
              });
              open = false;
            }}
            class="hover:bg-muted/50 flex w-full items-center justify-between rounded p-1.5 text-xs transition-colors {effectiveTx.categoryId ===
              cat.id && !effectiveTx.subcategoryId
              ? 'bg-muted/40 font-medium'
              : ''}"
          >
            <span class="flex items-center gap-1.5 truncate">
              <span
                class="size-2 shrink-0 rounded-full"
                style="background-color: {effectiveTypeColor};"
              ></span>
              <span class="truncate">{cat.name}</span>
            </span>
            {#if effectiveTx.categoryId === cat.id && !effectiveTx.subcategoryId}
              <CheckIcon class="size-3 shrink-0 text-emerald-500" />
            {/if}
          </button>
          {#if cat.subcategories.length > 0}
            <div class="space-y-0.5 pl-4">
              {#each cat.subcategories as sub (sub.id)}
                <button
                  type="button"
                  onclick={() => {
                    onDraftChange(txId, {
                      categoryId: cat.id as CategoryId,
                      typeId: txType.id as TypeId,
                      type: txType.name,
                      typeColor: txType.color,
                      subcategoryId: sub.id as SubcategoryId,
                    });
                    open = false;
                  }}
                  class="hover:bg-muted/50 text-muted-foreground hover:text-foreground flex w-full items-center justify-between rounded px-2 py-1 text-[11px] transition-colors {effectiveTx.categoryId ===
                    cat.id && effectiveTx.subcategoryId === sub.id
                    ? 'text-foreground font-medium'
                    : ''}"
                >
                  <span class="truncate">{sub.name}</span>
                  {#if effectiveTx.categoryId === cat.id && effectiveTx.subcategoryId === sub.id}
                    <CheckIcon class="size-3 shrink-0 text-emerald-500" />
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </Popover.Content>
</Popover.Root>
