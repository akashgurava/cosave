<script lang="ts">
  import CategorySankey from "./CategorySankey.svelte";
  import { categoryStore } from "$lib/categories";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import Loader2Icon from "@lucide/svelte/icons/loader-2";
  import LayersIcon from "@lucide/svelte/icons/layers";

  interface Props {
    activeFilter: string | string[];
    isAllSelected: boolean;
    onResetDefaults?: () => void;
  }

  let { activeFilter, isAllSelected, onResetDefaults }: Props = $props();

  async function handleReset() {
    if (onResetDefaults) {
      onResetDefaults();
    } else {
      await categoryStore.resetDefaults();
    }
  }
</script>

<div
  class="border-border/60 bg-card/60 relative h-180 min-h-180 w-full overflow-hidden rounded-xl border p-2 shadow-xs sm:h-200 sm:min-h-200 sm:p-4"
>
  {#if categoryStore.isLoading && !categoryStore.isLoaded}
    <div class="flex size-full flex-col items-center justify-center gap-3">
      <Loader2Icon class="text-primary size-8 animate-spin opacity-80" />
      <p class="text-muted-foreground text-sm font-medium">Loading category hierarchy...</p>
    </div>
  {:else if categoryStore.error}
    <div class="flex size-full flex-col items-center justify-center gap-3 p-6 text-center">
      <div
        class="bg-destructive/10 text-destructive flex size-12 items-center justify-center rounded-full"
      >
        <AlertCircleIcon class="size-6" />
      </div>
      <div class="space-y-1">
        <h3 class="text-base font-semibold">Failed to Load Categories</h3>
        <p class="text-muted-foreground max-w-sm text-sm">{categoryStore.error}</p>
      </div>
      <Button
        variant="outline"
        size="sm"
        class="mt-2 gap-1.5"
        onclick={() => void categoryStore.load()}
      >
        <RotateCcwIcon class="size-3.5" />
        <span>Try Again</span>
      </Button>
    </div>
  {:else if categoryStore.categories.length === 0}
    <div class="flex size-full flex-col items-center justify-center gap-4 p-6 text-center">
      <div class="bg-muted flex size-14 items-center justify-center rounded-2xl">
        <LayersIcon class="text-muted-foreground size-7" />
      </div>
      <div class="space-y-1.5">
        <h3 class="text-lg font-semibold tracking-tight">No Categories Found</h3>
        <p class="text-muted-foreground max-w-md text-sm">
          There are currently no transaction categories configured. Restore the default category
          structure to get started.
        </p>
      </div>
      <Button class="gap-2 shadow-xs" onclick={() => void handleReset()}>
        <RotateCcwIcon class="size-4" />
        <span>Restore Defaults</span>
      </Button>
    </div>
  {:else}
    {#if !isAllSelected}
      <div class="pointer-events-none absolute top-3 right-3 z-10">
        <Badge variant="secondary" class="text-xs backdrop-blur-xs">
          Filtered: {Array.isArray(activeFilter) ? activeFilter.join(", ") : activeFilter}
        </Badge>
      </div>
    {/if}
    <CategorySankey {activeFilter} />
  {/if}
</div>
