<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    open: boolean;
    title: string;
    description: string;
    confirmLabel?: string;
    onConfirm: () => void;
    onClose: () => void;
  }

  let { open, title, description, confirmLabel = "Delete", onConfirm, onClose }: Props = $props();

  function handleConfirm() {
    onConfirm();
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description>{description}</Dialog.Description>
    </Dialog.Header>

    <Dialog.Footer class="mt-4 flex flex-row justify-end gap-2">
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button variant="destructive" size="sm" onclick={handleConfirm}>
        {confirmLabel}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
