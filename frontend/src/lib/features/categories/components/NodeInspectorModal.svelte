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
  let quickCatError = $state<string | null>(null);
  let quickSubError = $state<string | null>(null);
  let isEditingName = $state(false);
  let editNameValue = $state("");
  let renameError = $state<string | null>(null);

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
    renameError = null;
    editNameValue = selectedNode.name;
    isEditingName = true;
  }

  async function saveRename() {
    renameError = null;
    if (!checkAuth()) return;
    if (!selectedNode) return;
    const trimmed = editNameValue.trim();
    if (!trimmed) {
      renameError = "Name cannot be empty.";
      return;
    }

    if (trimmed.toLowerCase() === selectedNode.name.toLowerCase()) {
      isEditingName = false;
      return;
    }

    if (selectedNode.kind === "category") {
      const alreadyExists = categoryStore.categories.some(
        (c) =>
          c.id !== selectedNode.id &&
          c.type.toLowerCase() === selectedNode.type.toLowerCase() &&
          c.name.toLowerCase() === trimmed.toLowerCase(),
      );
      if (alreadyExists) {
        renameError = `Category "${trimmed}" already exists under ${selectedNode.type}.`;
        return;
      }

      try {
        await categoryStore.renameCategory(selectedNode.id, trimmed);
        isEditingName = false;
      } catch (err) {
        renameError = err instanceof Error ? err.message : "Failed to rename category.";
      }
    } else if (selectedNode.kind === "subcategory" && selectedCategory) {
      const alreadyExists = selectedCategory.subcategories.some(
        (s) => s.id !== selectedNode.id && s.name.toLowerCase() === trimmed.toLowerCase(),
      );
      if (alreadyExists) {
        renameError = `Subcategory "${trimmed}" already exists under ${selectedCategory.name}.`;
        return;
      }

      try {
        await categoryStore.renameSubcategory(selectedNode.id, trimmed);
        isEditingName = false;
      } catch (err) {
        renameError = err instanceof Error ? err.message : "Failed to rename subcategory.";
      }
    }
  }

  async function handleAddQuickCategory() {
    quickCatError = null;
    if (!checkAuth()) return;
    if (!selectedNode || selectedNode.kind !== "type") return;
    const trimmed = quickCatName.trim();
    if (!trimmed) {
      quickCatError = "Please enter a category name.";
      return;
    }

    const alreadyExists = categoryStore.categories.some(
      (c) =>
        c.type.toLowerCase() === selectedNode.type.toLowerCase() &&
        c.name.toLowerCase() === trimmed.toLowerCase(),
    );
    if (alreadyExists) {
      quickCatError = `Category "${trimmed}" already exists under ${selectedNode.type}.`;
      return;
    }

    try {
      await categoryStore.addCategory(selectedNode.type, trimmed);
      quickCatName = "";
      quickCatError = null;
    } catch (err) {
      quickCatError = err instanceof Error ? err.message : `Failed to add category "${trimmed}".`;
    }
  }

  async function handleAddQuickSubcategory() {
    quickSubError = null;
    if (!checkAuth()) return;
    if (!selectedCategory) return;
    const trimmed = quickSubName.trim();
    if (!trimmed) {
      quickSubError = "Please enter a subcategory name.";
      return;
    }

    const alreadyExists = selectedCategory.subcategories.some(
      (s) => s.name.toLowerCase() === trimmed.toLowerCase(),
    );
    if (alreadyExists) {
      quickSubError = `Subcategory "${trimmed}" already exists under ${selectedCategory.name}.`;
      return;
    }

    try {
      await categoryStore.addSubcategory(selectedCategory.id, trimmed);
      quickSubName = "";
      quickSubError = null;
    } catch (err) {
      quickSubError =
        err instanceof Error ? err.message : `Failed to add subcategory "${trimmed}".`;
    }
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
    quickCatError = null;
    quickSubError = null;
    renameError = null;
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && handleModalClose()}>
  <Dialog.Content class="sm:max-w-lg">
    {#if selectedNode}
      {@const nodeColor = categoryStore.getTypeColor(selectedNode.type)}
      <Dialog.Header class="space-y-3 pr-8">
        {#if !authStore.isAuthenticated}
          <div
            class="border-border/60 bg-muted/40 text-muted-foreground flex items-center justify-between rounded-lg border px-3 py-1.5 text-xs"
          >
            <span>Read-only preview. Sign in to edit or delete.</span>
            {#if onRequireAuth}
              <Button
                variant="outline"
                size="sm"
                class="h-7 px-2 text-xs font-medium"
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
            <div class="flex flex-1 flex-col gap-1.5">
              <div class="flex items-center gap-2">
                <Input
                  bind:value={editNameValue}
                  aria-label="New name"
                  class="h-9 font-medium"
                  placeholder="Enter new name"
                  onkeydown={(e) => e.key === "Enter" && saveRename()}
                  oninput={() => (renameError = null)}
                />
                <Button size="sm" class="h-9 text-xs" aria-label="Save name" onclick={saveRename}
                  >Save</Button
                >
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-9 text-xs"
                  aria-label="Cancel editing"
                  onclick={() => {
                    isEditingName = false;
                    renameError = null;
                  }}
                >
                  Cancel
                </Button>
              </div>
              {#if renameError}
                <p class="text-destructive text-xs font-medium">{renameError}</p>
              {/if}
            </div>
          {:else}
            <div class="flex flex-wrap items-center gap-2.5">
              <Dialog.Title class="text-foreground text-xl font-bold tracking-tight">
                {selectedNode.name}
              </Dialog.Title>
              <Badge variant="secondary" class="text-xs font-semibold tracking-wider uppercase">
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
                  aria-label={`Rename ${selectedNode.name}`}
                  title="Rename"
                >
                  <Edit3Icon class="size-4" />
                </Button>
              {/if}
              <Button
                variant="ghost"
                size="icon"
                class="text-destructive hover:bg-destructive/10 hover:text-destructive size-8"
                onclick={handleDeleteCurrentNode}
                aria-label={`Delete ${selectedNode.kind} ${selectedNode.name}`}
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
                {categoryStore.colors.find(
                  (c) => c.hex.toLowerCase() === selectedTypeItem.color.toLowerCase(),
                )?.name ?? "Custom"}
              </span>
            </div>

            <div class="flex flex-wrap items-center gap-2 pt-1">
              {#each categoryStore.colors as color (color.id)}
                {@const isCurrent =
                  selectedTypeItem.color.toLowerCase() === color.hex.toLowerCase()}
                {@const isUsedByOther = categoryStore.isColorUsed(color.hex, selectedTypeItem.name)}
                <button
                  type="button"
                  disabled={isUsedByOther}
                  aria-label={isUsedByOther
                    ? `${color.name} (in use)`
                    : `Select color ${color.name}`}
                  class={`relative flex size-8 items-center justify-center rounded-full transition-transform ${
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
                    <CheckIcon class="size-3.5 text-white drop-shadow-xs" />
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
            <div class="space-y-1.5">
              <div class="flex items-center gap-2">
                <Input
                  bind:value={quickCatName}
                  aria-label="New category name"
                  placeholder="Add category (e.g. Utilities)..."
                  class="h-9 text-xs"
                  onkeydown={(e) => e.key === "Enter" && handleAddQuickCategory()}
                  oninput={() => (quickCatError = null)}
                />
                <Button
                  size="sm"
                  class="h-9 shrink-0 gap-1 px-3 text-xs"
                  onclick={handleAddQuickCategory}
                >
                  <PlusIcon class="size-3.5" />
                  <span>Add</span>
                </Button>
              </div>
              {#if quickCatError}
                <p class="text-destructive text-xs font-medium">{quickCatError}</p>
              {/if}
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
                          categoryId: null,
                        });
                      }}
                    >
                      <span>{cat.name}</span>
                      <span class="text-muted-foreground text-xs font-normal">
                        ({cat.subcategories.length}
                        {cat.subcategories.length === 1 ? "sub" : "subs"})
                      </span>
                    </button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="text-muted-foreground hover:bg-destructive/10 hover:text-destructive size-8"
                      onclick={() => categoryStore.deleteCategory(cat.id)}
                      aria-label={`Delete category ${cat.name}`}
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
              class="border-destructive/30 text-destructive hover:bg-destructive/10 hover:text-destructive h-8 gap-1 text-xs"
              onclick={handleDeleteCurrentNode}
              aria-label="Delete transaction type"
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
            <div class="space-y-1.5">
              <div class="flex items-center gap-2">
                <Input
                  bind:value={quickSubName}
                  aria-label="New subcategory name"
                  placeholder="Add subcategory (e.g. Fuel, Index ETFs)..."
                  class="h-9 text-xs"
                  onkeydown={(e) => e.key === "Enter" && handleAddQuickSubcategory()}
                  oninput={() => (quickSubError = null)}
                />
                <Button
                  size="sm"
                  class="h-9 shrink-0 gap-1 px-3 text-xs"
                  onclick={handleAddQuickSubcategory}
                >
                  <PlusIcon class="size-3.5" />
                  <span>Add</span>
                </Button>
              </div>
              {#if quickSubError}
                <p class="text-destructive text-xs font-medium">{quickSubError}</p>
              {/if}
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
                      class="text-muted-foreground hover:bg-destructive/10 hover:text-destructive size-8"
                      onclick={() => {
                        if (selectedCategory) {
                          categoryStore.deleteSubcategory(selectedCategory.id, sub.id);
                        }
                      }}
                      aria-label={`Delete subcategory ${sub.name}`}
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
                    categoryId: null,
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
