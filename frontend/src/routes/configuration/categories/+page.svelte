<script lang="ts">
  import { onMount } from "svelte";

  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";

  import { Button } from "$lib/components/ui/button";

  import {
    AddTypeModal,
    CategoryFilterBar,
    CategorySankeyCard,
    categoryStore,
    NodeInspectorModal,
    ResetDefaultsModal,
  } from "$lib/features/categories";

  import type { TypeId } from "$lib/types";

  let selectedTypeIds = $state<TypeId[]>([]);
  let isAddTypeOpen = $state(false);
  let isResetConfirmOpen = $state(false);

  onMount(() => {
    void categoryStore.load();
  });

  function handleAddTypeClick() {
    isAddTypeOpen = true;
  }

  function handleResetDefaultsClick() {
    isResetConfirmOpen = true;
  }

  const isAllSelected = $derived(
    selectedTypeIds.length === 0 || selectedTypeIds.length === categoryStore.types.length,
  );

  function toggleType(typeId: TypeId) {
    if (isAllSelected === true) {
      selectedTypeIds = categoryStore.types.filter((t) => t.id !== typeId).map((t) => t.id);
      return;
    }
    const exists = selectedTypeIds.includes(typeId);
    if (exists === true) {
      const next = selectedTypeIds.filter((id) => id !== typeId);
      selectedTypeIds = next.length === 0 ? [] : next;
    } else {
      const next = [...selectedTypeIds, typeId];
      selectedTypeIds = next.length === categoryStore.types.length ? [] : next;
    }
  }

  function selectOnly(typeId: TypeId) {
    selectedTypeIds = [typeId];
  }

  function selectAll() {
    selectedTypeIds = [];
  }
</script>

<svelte:head>
  <title>Transaction Hierarchy &bull; CoSave</title>
</svelte:head>

<div class="flex h-full min-h-0 flex-1 flex-col gap-4">
  <!-- Breadcrumb Navigation Bar -->
  <div class="text-muted-foreground flex shrink-0 items-center gap-2 text-xs">
    <span class="text-muted-foreground">Configuration</span>
    <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
    <span class="text-foreground font-medium">Transaction Hierarchy</span>
  </div>

  <!-- Page Header -->
  <div class="flex shrink-0 flex-col justify-between gap-4 md:flex-row md:items-center">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Transaction Hierarchy</h1>
    </div>

    <!-- Header Actions -->
    <div class="flex items-center gap-2.5">
      <Button
        variant="outline"
        size="sm"
        onclick={handleResetDefaultsClick}
        title="Reset categories to defaults"
      >
        <RotateCcwIcon class="size-3.5" />
        <span>Reset Defaults</span>
      </Button>

      <Button size="sm" onclick={handleAddTypeClick}>
        <PlusIcon class="size-4" />
        <span>Add Transaction Type</span>
      </Button>
    </div>
  </div>

  <!-- Filter Pills & Statistics Bar -->
  <div class="shrink-0">
    <CategoryFilterBar
      {selectedTypeIds}
      onToggleType={toggleType}
      onSelectOnly={selectOnly}
      onSelectAll={selectAll}
    />
  </div>

  <!-- Interactive Sankey Card (expands to fill remaining container height) -->
  <CategorySankeyCard
    {selectedTypeIds}
    {isAllSelected}
    onResetDefaults={handleResetDefaultsClick}
  />
</div>

<!-- Modal Dialogs -->
<NodeInspectorModal
  open={categoryStore.selectedNode !== null}
  onClose={() => categoryStore.setSelectedNode(null)}
/>
<AddTypeModal open={isAddTypeOpen} onClose={() => (isAddTypeOpen = false)} />
<ResetDefaultsModal open={isResetConfirmOpen} onClose={() => (isResetConfirmOpen = false)} />
