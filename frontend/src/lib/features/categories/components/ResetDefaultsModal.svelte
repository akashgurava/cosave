<script lang="ts">
  import { categoryStore } from "../store";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import AlertTriangleIcon from "@lucide/svelte/icons/alert-triangle";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  async function handleReset() {
    await categoryStore.resetDefaults();
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <div class="text-destructive flex items-center gap-2">
        <AlertTriangleIcon class="size-5" />
        <Dialog.Title class="text-foreground">Reset Defaults?</Dialog.Title>
      </div>
      <Dialog.Description class="pt-2 text-xs leading-relaxed">
        This will restore all default transaction types (Income, Expense, Transfer, Invest),
        categories, and subcategories to standard defaults. Any custom types, categories, or color
        adjustments will be reset.
      </Dialog.Description>
    </Dialog.Header>

    <Dialog.Footer class="pt-2">
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button variant="destructive" size="sm" onclick={handleReset}>Yes, Reset Defaults</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
