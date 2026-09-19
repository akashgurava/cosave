<script lang="ts">
  import { onMount } from "svelte";
  import {
    categoryStore,
    CategoryFilterBar,
    CategorySankeyCard,
    AddTypeModal,
    ResetDefaultsModal,
    NodeInspectorModal,
  } from "$lib/features/categories";
  import { authStore, AuthModal } from "$lib/features/auth";
  import { Button } from "$lib/components/ui/button";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  let selectedTypes = $state<string[]>([]);
  let isAddTypeOpen = $state(false);
  let isResetConfirmOpen = $state(false);
  let isAuthModalOpen = $state(false);

  onMount(() => {
    void categoryStore.load();
  });

  function handleRequireAuth() {
    isAuthModalOpen = true;
  }

  function handleAddTypeClick() {
    if (!authStore.isAuthenticated) {
      isAuthModalOpen = true;
      return;
    }
    isAddTypeOpen = true;
  }

  function handleResetDefaultsClick() {
    if (!authStore.isAuthenticated) {
      isAuthModalOpen = true;
      return;
    }
    isResetConfirmOpen = true;
  }

  const isAllSelected = $derived(
    selectedTypes.length === 0 || selectedTypes.length === categoryStore.types.length,
  );

  const activeFilter = $derived<string | string[]>(
    isAllSelected ? "All" : selectedTypes.length === 1 ? selectedTypes[0] : selectedTypes,
  );

  function toggleType(typeName: string) {
    if (isAllSelected) {
      selectedTypes = categoryStore.types
        .filter((t) => t.name.toLowerCase() !== typeName.toLowerCase())
        .map((t) => t.name);
      return;
    }
    const exists = selectedTypes.some((t) => t.toLowerCase() === typeName.toLowerCase());
    if (exists) {
      const next = selectedTypes.filter((t) => t.toLowerCase() !== typeName.toLowerCase());
      selectedTypes = next.length === 0 ? [] : next;
    } else {
      const next = [...selectedTypes, typeName];
      selectedTypes = next.length === categoryStore.types.length ? [] : next;
    }
  }

  function selectOnly(typeName: string) {
    selectedTypes = [typeName];
  }

  function selectAll() {
    selectedTypes = [];
  }
</script>

<svelte:head>
  <title>Transaction Hierarchy &bull; CoSave</title>
</svelte:head>

<div class="space-y-6">
  <!-- Breadcrumb Navigation Bar -->
  <div class="text-muted-foreground flex items-center gap-2 text-xs">
    <span class="text-muted-foreground">Configuration</span>
    <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
    <span class="text-foreground font-medium">Transaction Hierarchy</span>
  </div>

  <!-- Page Header -->
  <div class="flex flex-col justify-between gap-4 md:flex-row md:items-center">
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
  <CategoryFilterBar
    {selectedTypes}
    onToggleType={toggleType}
    onSelectOnly={selectOnly}
    onSelectAll={selectAll}
  />

  <!-- Interactive Sankey Card -->
  <CategorySankeyCard {activeFilter} {isAllSelected} onResetDefaults={handleResetDefaultsClick} />
</div>

<!-- Modal Dialogs -->
<NodeInspectorModal
  open={categoryStore.selectedNode !== null}
  onClose={() => categoryStore.setSelectedNode(null)}
  onRequireAuth={handleRequireAuth}
/>
<AddTypeModal open={isAddTypeOpen} onClose={() => (isAddTypeOpen = false)} />
<ResetDefaultsModal open={isResetConfirmOpen} onClose={() => (isResetConfirmOpen = false)} />
<AuthModal isOpen={isAuthModalOpen} onClose={() => (isAuthModalOpen = false)} />
