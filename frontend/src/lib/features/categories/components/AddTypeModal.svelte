<script lang="ts">
  import { categoryStore } from "../store.svelte";
  import { isSameColor, type Color } from "../types";
  import { getTextFormValue } from "$lib/types";
  import { isNull } from "$lib/api";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import CheckIcon from "@lucide/svelte/icons/check";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let selectedColor = $state<Color>(categoryStore.getUnassignedColor());
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open === true) {
      errorMessage = null;
      selectedColor = categoryStore.getUnassignedColor();
    }
  });

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    errorMessage = null;
    const form = e.currentTarget as HTMLFormElement;
    const formData = new FormData(form);
    const name = getTextFormValue(formData, "name");

    try {
      await categoryStore.addType(name, selectedColor.id);
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : "Failed to create transaction type.";
      return;
    }

    form.reset();
    onClose();
  }
</script>

<Dialog.Root
  {open}
  onOpenChange={(isOpen) => {
    if (isOpen === false) onClose();
  }}
>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Add Transaction Type</Dialog.Title>
      <Dialog.Description class="sr-only">
        Create a new root financial classification level with a distinct color.
      </Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSubmit}>
      <div class="space-y-4 py-2">
        <!-- Type Name Input -->
        <div class="space-y-1.5">
          <label for="type-name-input" class="text-muted-foreground text-xs font-semibold">
            Name
          </label>
          <Input
            id="type-name-input"
            name="name"
            placeholder="e.g. Savings, Debt, Liability"
            oninput={() => (errorMessage = null)}
          />
        </div>

        <!-- Color Selection Palette -->
        <div class="space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground font-semibold">Color</span>
            <span class="text-muted-foreground text-xs">{selectedColor.name}</span>
          </div>

          <div class="grid grid-cols-6 gap-2 pt-1">
            {#each categoryStore.colors as color (color.id)}
              {@const isUsed = categoryStore.isColorUsed(color)}
              {@const isSelected = isSameColor(selectedColor, color)}
              <button
                type="button"
                disabled={isUsed}
                aria-label={isUsed
                  ? `${color.name} (already in use)`
                  : `Select color ${color.name}`}
                class={`relative flex size-9 items-center justify-center rounded-lg border transition-all ${
                  isSelected
                    ? "border-foreground ring-foreground scale-105 shadow-sm ring-2 ring-offset-2"
                    : isUsed
                      ? "cursor-not-allowed border-transparent opacity-20"
                      : "cursor-pointer border-transparent hover:scale-105 hover:shadow-xs"
                }`}
                style="background-color: {color.hex};"
                onclick={() => (selectedColor = color)}
                title={isUsed ? `${color.name} (already in use)` : color.name}
              >
                {#if isSelected}
                  <CheckIcon class="size-4 text-white drop-shadow-xs" />
                {/if}
              </button>
            {/each}
          </div>
        </div>

        {#if !isNull(errorMessage)}
          <p class="text-destructive text-xs font-medium">{errorMessage}</p>
        {/if}
      </div>

      <Dialog.Footer class="pt-4">
        <Button variant="outline" size="sm" type="button" onclick={onClose}>Cancel</Button>
        <Button size="sm" type="submit">Create Type</Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
