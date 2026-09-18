<script lang="ts">
  import { PRESET_COLORS, categoryStore } from "$lib/categories";
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
      const firstAvailable = PRESET_COLORS.find((c) => !categoryStore.isColorUsed(c.hex));
      typeColor = firstAvailable ? firstAvailable.hex : PRESET_COLORS[0].hex;
    }
  });

  async function handleCreate() {
    errorMessage = null;
    const trimmed = typeName.trim();
    if (!trimmed) {
      errorMessage = "Please enter a transaction type name.";
      return;
    }

    const created = await categoryStore.addType(trimmed, typeColor);
    if (!created) {
      errorMessage = `Failed to create type "${trimmed}" or it already exists.`;
      return;
    }

    onClose();
    // Auto-select newly created type to open inspector
    categoryStore.setSelectedNode({
      id: `type:${created.name}`,
      kind: "type",
      type: created.name,
      name: created.name,
    });
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-105">
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
        />
      </div>

      <!-- Color Selection Palette -->
      <div class="space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="text-muted-foreground font-semibold">Select Color</span>
          <span class="text-muted-foreground text-[11px]">
            {PRESET_COLORS.find((c) => c.hex.toLowerCase() === typeColor.toLowerCase())?.name ??
              "Custom"}
          </span>
        </div>

        <div class="grid grid-cols-6 gap-2 pt-1">
          {#each PRESET_COLORS as color (color.id)}
            {@const isUsed = categoryStore.isColorUsed(color.hex)}
            {@const isSelected = typeColor.toLowerCase() === color.hex.toLowerCase()}
            <button
              type="button"
              disabled={isUsed}
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
        <p class="text-xs font-medium text-rose-500">{errorMessage}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button
        size="sm"
        class="bg-emerald-600 text-white shadow-sm hover:bg-emerald-500"
        onclick={handleCreate}
      >
        Create Type
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
