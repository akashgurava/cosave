<script lang="ts">
  import { categoryStore } from "../store";
  import { PRESET_COLORS } from "../types";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import CheckIcon from "@lucide/svelte/icons/check";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let typeName = $state("");
  let typeColor = $state<string>("");
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open) {
      typeName = "";
      errorMessage = null;
      const firstAvailable = categoryStore.colors.find((c) => !categoryStore.isColorUsed(c.hex));
      typeColor = firstAvailable ? firstAvailable.hex : (categoryStore.colors[0]?.hex ?? "#10b981");
    }
  });

  async function handleCreate() {
    errorMessage = null;
    const trimmed = typeName.trim();
    if (!trimmed) {
      errorMessage = "Please enter a transaction type name.";
      return;
    }

    const alreadyExists = categoryStore.types.some(
      (t) => t.name.toLowerCase() === trimmed.toLowerCase(),
    );
    if (alreadyExists) {
      errorMessage = `Transaction type "${trimmed}" already exists.`;
      return;
    }

    try {
      const created = await categoryStore.addType(trimmed, typeColor);
      if (!created) {
        errorMessage = `Failed to create type "${trimmed}".`;
        return;
      }

      onClose();
      // Auto-select newly created type to open inspector
      categoryStore.setSelectedNode({
        id: `type:${created.name}`,
        kind: "type",
        type: created.name,
        name: created.name,
        parentName: null,
        categoryId: null,
      });
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : `Failed to create type "${trimmed}".`;
    }
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Add Transaction Type</Dialog.Title>
      <Dialog.Description>
        Create a new root financial classification level with a distinct color.
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-2">
      <!-- Type Name Input -->
      <div class="space-y-1.5">
        <label for="type-name-input" class="text-muted-foreground text-xs font-semibold">
          Type Name
        </label>
        <Input
          id="type-name-input"
          bind:value={typeName}
          placeholder="e.g. Savings, Debt, Liability"
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
          oninput={() => (errorMessage = null)}
        />
      </div>

      <!-- Color Selection Palette -->
      <div class="space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-muted-foreground font-semibold">Select Color</span>
          <span class="text-muted-foreground text-[11px]">
            {categoryStore.colors.find((c) => c.hex.toLowerCase() === typeColor.toLowerCase())
              ?.name ?? "Custom"}
          </span>
        </div>

        <div class="grid grid-cols-6 gap-2 pt-1">
          {#each categoryStore.colors as color (color.id)}
            {@const isUsed = categoryStore.isColorUsed(color.hex)}
            {@const isSelected = typeColor.toLowerCase() === color.hex.toLowerCase()}
            <button
              type="button"
              disabled={isUsed}
              aria-label={isUsed ? `${color.name} (already in use)` : `Select color ${color.name}`}
              class={`relative flex size-9 items-center justify-center rounded-lg border transition-all ${
                isSelected
                  ? "border-foreground ring-foreground scale-105 shadow-sm ring-2 ring-offset-2"
                  : isUsed
                    ? "cursor-not-allowed border-transparent opacity-20"
                    : "cursor-pointer border-transparent hover:scale-105 hover:shadow-xs"
              }`}
              style="background-color: {color.hex};"
              onclick={() => (typeColor = color.hex)}
              title={isUsed ? `${color.name} (already in use)` : color.name}
            >
              {#if isSelected}
                <CheckIcon class="size-4 text-white drop-shadow-xs" />
              {/if}
            </button>
          {/each}
        </div>
      </div>

      {#if errorMessage}
        <p class="text-destructive text-xs font-medium">{errorMessage}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button size="sm" onclick={handleCreate}>Create Type</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
