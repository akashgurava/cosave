<script lang="ts">
  import { categoryStore } from "../store";
  import { PRESET_COLORS, type CategoryItem, type TransactionTypeItem } from "../types";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import Edit3Icon from "@lucide/svelte/icons/edit-3";
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import CheckIcon from "@lucide/svelte/icons/check";
  import PaletteIcon from "@lucide/svelte/icons/palette";

  import { authStore } from "$lib/features/auth";

  interface Props {
    open: boolean;
    onClose: () => void;
    onRequireAuth?: () => void;
  }

  let { open, onClose, onRequireAuth }: Props = $props();

  let quickCatName = $state("");
  let quickSubName = $state("");
  let isEditingName = $state(false);
  let editNameValue = $state("");

  function checkAuth(): boolean {
    if (!authStore.isAuthenticated) {
      if (onRequireAuth) onRequireAuth();
      return false;
    }
    return true;
  }

  const selectedNode = $derived(categoryStore.selectedNode);

  const selectedTypeItem = $derived.by<TransactionTypeItem | null>(() => {
    if (!selectedNode || selectedNode.kind !== "type") return null;
    return categoryStore.getType(selectedNode.type) ?? null;
  });

  const categoriesUnderSelectedType = $derived.by<CategoryItem[]>(() => {
    if (!selectedNode || selectedNode.kind !== "type") return [];
    return categoryStore.categories.filter(
      (c) => c.type.toLowerCase() === selectedNode.type.toLowerCase(),
    );
  });

  const selectedCategory = $derived.by<CategoryItem | null>(() => {
    if (!selectedNode) return null;
    if (selectedNode.kind === "category") {
      return categoryStore.categories.find((c) => c.id === selectedNode.id) ?? null;
    }
    if (selectedNode.kind === "subcategory" && selectedNode.categoryId) {
      return categoryStore.categories.find((c) => c.id === selectedNode.categoryId) ?? null;
    }
    return null;
  });

  function startRename() {
    if (!checkAuth()) return;
    if (!selectedNode) return;
    editNameValue = selectedNode.name;
    isEditingName = true;
  }

  async function saveRename() {
    if (!checkAuth()) return;
    if (!selectedNode) return;
    const trimmed = editNameValue.trim();
    if (!trimmed) {
      isEditingName = false;
      return;
    }

    if (selectedNode.kind === "category") {
      await categoryStore.renameCategory(selectedNode.id, trimmed);
    } else if (selectedNode.kind === "subcategory") {
      await categoryStore.renameSubcategory(selectedNode.id, trimmed);
    }
    isEditingName = false;
  }

  async function handleAddQuickCategory() {
    if (!checkAuth()) return;
    if (!selectedNode || selectedNode.kind !== "type") return;
    const trimmed = quickCatName.trim();
    if (!trimmed) return;

    await categoryStore.addCategory(selectedNode.type, trimmed);
    quickCatName = "";
  }

  async function handleAddQuickSubcategory() {
    if (!checkAuth()) return;
    if (!selectedCategory) return;
    const trimmed = quickSubName.trim();
    if (!trimmed) return;

    await categoryStore.addSubcategory(selectedCategory.id, trimmed);
    quickSubName = "";
  }

  async function handleDeleteCurrentNode() {
    if (!checkAuth()) return;
    if (!selectedNode) return;

    if (selectedNode.kind === "type") {
      await categoryStore.deleteType(selectedNode.type);
    } else if (selectedNode.kind === "category") {
      await categoryStore.deleteCategory(selectedNode.id);
    } else if (selectedNode.kind === "subcategory") {
      await categoryStore.deleteSubcategory(selectedNode.id);
    }
    onClose();
  }

  async function handleTypeColorChange(newColor: string) {
    if (!checkAuth()) return;
    if (!selectedNode || selectedNode.kind !== "type") return;
    await categoryStore.updateTypeColor(selectedNode.type, newColor);
  }

  function handleModalClose() {
    categoryStore.setSelectedNode(null);
    isEditingName = false;
    quickCatName = "";
    quickSubName = "";
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && handleModalClose()}>
  <Dialog.Content class="sm:max-w-115">
    {#if selectedNode}
      {@const nodeColor = categoryStore.getTypeColor(selectedNode.type)}
      <Dialog.Header class="space-y-3 pr-8">
        {#if !authStore.isAuthenticated}
          <div
            class="flex items-center justify-between rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-1.5 text-xs text-amber-700 dark:text-amber-400"
          >
            <span>Read-only preview. Sign in to edit or delete.</span>
            {#if onRequireAuth}
              <Button
                variant="outline"
                size="sm"
                class="h-6 border-amber-500/30 px-2 text-[11px] font-medium"
                onclick={onRequireAuth}
              >
                Sign In
              </Button>
            {/if}
          </div>
        {/if}

        <!-- Breadcrumb Path -->
        <div class="text-muted-foreground flex flex-wrap items-center gap-1.5 text-xs">
          <span class="text-foreground flex items-center gap-1 font-medium">
            <span class="size-2 rounded-full" style="background-color: {nodeColor.solid};"></span>
            {selectedNode.type}
          </span>

          {#if selectedNode.kind === "category"}
            <ArrowRightIcon class="text-muted-foreground/50 size-3" />
            <span class="text-foreground font-medium">{selectedNode.name}</span>
          {:else if selectedNode.kind === "subcategory"}
            <ArrowRightIcon class="text-muted-foreground/50 size-3" />
            <span>{selectedNode.parentName}</span>
            <ArrowRightIcon class="text-muted-foreground/50 size-3" />
            <span class="text-foreground font-medium">{selectedNode.name}</span>
          {/if}
        </div>

        <!-- Title & Actions Row -->
        <div class="flex items-center justify-between gap-3">
          {#if isEditingName}
            <div class="flex flex-1 items-center gap-2">
              <Input
                bind:value={editNameValue}
                class="h-9 font-medium"
                placeholder="Enter new name"
                onkeydown={(e) => e.key === "Enter" && saveRename()}
              />
              <Button size="sm" class="h-9 text-xs" onclick={saveRename}>Save</Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-9 text-xs"
                onclick={() => (isEditingName = false)}
              >
                Cancel
              </Button>
            </div>
          {:else}
            <div class="flex flex-wrap items-center gap-2.5">
              <Dialog.Title class="text-foreground text-xl font-bold tracking-tight">
                {selectedNode.name}
              </Dialog.Title>
              <Badge variant="secondary" class="text-[10px] font-semibold tracking-wider uppercase">
                {selectedNode.kind}
              </Badge>
            </div>

            <div class="flex items-center gap-1">
              {#if selectedNode.kind !== "type"}
                <Button
                  variant="ghost"
                  size="icon"
                  class="text-muted-foreground hover:text-foreground size-8"
                  onclick={startRename}
                  title="Rename"
                >
                  <Edit3Icon class="size-4" />
                </Button>
              {/if}
              <Button
                variant="ghost"
                size="icon"
                class="size-8 text-rose-500 hover:bg-rose-500/10 hover:text-rose-600"
                onclick={handleDeleteCurrentNode}
                title={`Delete ${selectedNode.kind}`}
              >
                <Trash2Icon class="size-4" />
              </Button>
            </div>
          {/if}
        </div>
      </Dialog.Header>

      <div class="space-y-4 pt-1">
        <!-- KIND: TYPE -->
        {#if selectedNode.kind === "type" && selectedTypeItem}
          <!-- Color Switcher for Type -->
          <div class="border-border/60 bg-muted/20 space-y-2 rounded-lg border p-3">
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground flex items-center gap-1.5 font-medium">
                <PaletteIcon class="size-3.5" /> Type Color
              </span>
              <span class="text-muted-foreground text-[11px]">
                {PRESET_COLORS.find(
                  (c) => c.hex.toLowerCase() === selectedTypeItem.color.toLowerCase(),
                )?.name ?? "Custom"}
              </span>
            </div>

            <div class="flex flex-wrap items-center gap-2 pt-1">
              {#each PRESET_COLORS as color (color.id)}
                {@const isCurrent =
                  selectedTypeItem.color.toLowerCase() === color.hex.toLowerCase()}
                {@const isUsedByOther = categoryStore.isColorUsed(color.hex, selectedTypeItem.name)}
                <button
                  type="button"
                  disabled={isUsedByOther}
                  class={`relative flex size-6.5 items-center justify-center rounded-full transition-transform ${
                    isCurrent
                      ? "ring-foreground scale-110 shadow-sm ring-2 ring-offset-2"
                      : isUsedByOther
                        ? "cursor-not-allowed opacity-25"
                        : "cursor-pointer hover:scale-110"
                  }`}
                  style="background-color: {color.hex};"
                  onclick={() => handleTypeColorChange(color.hex)}
                  title={isUsedByOther ? `${color.name} (in use)` : color.name}
                >
                  {#if isCurrent}
                    <CheckIcon class="size-3 text-white drop-shadow-xs" />
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- Categories under this Type -->
          <div class="space-y-3 pt-1">
            <div
              class="text-muted-foreground flex items-center justify-between text-xs font-semibold"
            >
              <span
                >Categories under {selectedNode.type} ({categoriesUnderSelectedType.length})</span
              >
            </div>

            <!-- Add quick category input -->
            <div class="flex items-center gap-2">
              <Input
                bind:value={quickCatName}
                placeholder="Add category (e.g. Utilities)..."
                class="h-9 text-xs"
                onkeydown={(e) => e.key === "Enter" && handleAddQuickCategory()}
              />
              <Button
                size="sm"
                class="h-9 shrink-0 gap-1 bg-emerald-600 px-3 text-xs text-white shadow-xs hover:bg-emerald-500"
                onclick={handleAddQuickCategory}
              >
                <PlusIcon class="size-3.5" />
                <span>Add</span>
              </Button>
            </div>

            <!-- Categories List under this type -->
            {#if categoriesUnderSelectedType.length > 0}
              <div class="max-h-52 space-y-1.5 overflow-y-auto pr-0.5">
                {#each categoriesUnderSelectedType as cat (cat.id)}
                  <div
                    class="group border-border/60 bg-muted/30 hover:bg-muted/60 flex items-center justify-between rounded-md border px-3 py-2 text-xs transition-colors"
                  >
                    <button
                      type="button"
                      class="text-foreground flex cursor-pointer items-center gap-2 text-left font-medium hover:underline"
                      onclick={() => {
                        categoryStore.setSelectedNode({
                          id: cat.id,
                          kind: "category",
                          type: cat.type,
                          name: cat.name,
                          parentName: cat.type,
                        });
                      }}
                    >
                      <span>{cat.name}</span>
                      <span class="text-muted-foreground text-[11px] font-normal">
                        ({cat.subcategories.length}
                        {cat.subcategories.length === 1 ? "sub" : "subs"})
                      </span>
                    </button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground size-6 hover:text-rose-500"
                      onclick={() => categoryStore.deleteCategory(cat.id)}
                      title="Delete category"
                    >
                      <Trash2Icon class="size-3.5" />
                    </Button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="text-muted-foreground/80 py-2 text-xs italic">
                No categories defined under {selectedNode.type} yet. Add one above.
              </p>
            {/if}
          </div>

          <!-- Delete Type Section -->
          <div class="border-border/60 flex items-center justify-between border-t pt-3">
            <span class="text-muted-foreground text-xs">Delete this entire transaction type</span>
            <Button
              variant="outline"
              size="sm"
              class="h-8 gap-1 border-rose-500/30 text-xs text-rose-500 hover:bg-rose-500/10 hover:text-rose-600"
              onclick={handleDeleteCurrentNode}
            >
              <Trash2Icon class="size-3.5" />
              <span>Delete Type</span>
            </Button>
          </div>

          <!-- KIND: CATEGORY -->
        {:else if selectedNode.kind === "category" && selectedCategory}
          <div class="space-y-3">
            <div
              class="text-muted-foreground flex items-center justify-between text-xs font-semibold"
            >
              <span>Subcategories ({selectedCategory.subcategories.length})</span>
            </div>

            <!-- Add quick subcategory input -->
            <div class="flex items-center gap-2">
              <Input
                bind:value={quickSubName}
                placeholder="Add subcategory (e.g. Fuel, Index ETFs)..."
                class="h-9 text-xs"
                onkeydown={(e) => e.key === "Enter" && handleAddQuickSubcategory()}
              />
              <Button
                size="sm"
                class="h-9 shrink-0 gap-1 bg-emerald-600 px-3 text-xs text-white shadow-xs hover:bg-emerald-500"
                onclick={handleAddQuickSubcategory}
              >
                <PlusIcon class="size-3.5" />
                <span>Add</span>
              </Button>
            </div>

            <!-- Subcategories List -->
            {#if selectedCategory.subcategories.length > 0}
              <div class="max-h-52 space-y-1.5 overflow-y-auto pr-0.5">
                {#each selectedCategory.subcategories as sub (sub.id)}
                  <div
                    class="group border-border/60 bg-muted/30 hover:bg-muted/60 flex items-center justify-between rounded-md border px-3 py-2 text-xs transition-colors"
                  >
                    <span class="text-foreground font-medium">{sub.name}</span>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground size-6 hover:text-rose-500"
                      onclick={() => {
                        if (selectedCategory) {
                          categoryStore.deleteSubcategory(selectedCategory.id, sub.id);
                        }
                      }}
                      title="Delete subcategory"
                    >
                      <Trash2Icon class="size-3.5" />
                    </Button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="text-muted-foreground/80 py-2 text-xs italic">
                No subcategories defined yet. Transactions will map directly to {selectedCategory.name}.
              </p>
            {/if}
          </div>

          <!-- KIND: SUBCATEGORY -->
        {:else if selectedNode.kind === "subcategory"}
          <div class="bg-muted/30 space-y-3 rounded-md p-3">
            <p class="text-muted-foreground text-xs">
              Leaf subcategory grouped under <strong class="text-foreground"
                >{selectedNode.parentName}</strong
              >.
            </p>
            <Button
              variant="outline"
              size="sm"
              class="w-full justify-start text-xs"
              onclick={() => {
                if (selectedCategory) {
                  categoryStore.setSelectedNode({
                    id: selectedCategory.id,
                    kind: "category",
                    type: selectedCategory.type,
                    name: selectedCategory.name,
                    parentName: selectedCategory.type,
                  });
                }
              }}
            >
              <ArrowRightIcon class="mr-1.5 size-3.5 rotate-180" />
              <span>Back to Parent Category ({selectedNode.parentName})</span>
            </Button>
          </div>
        {/if}
      </div>

      <Dialog.Footer>
        <Button size="sm" variant="outline" onclick={handleModalClose}>Done</Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
