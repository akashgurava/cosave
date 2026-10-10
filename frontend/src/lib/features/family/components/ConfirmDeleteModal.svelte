<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    open: boolean;
    title: string;
    description: string;
    confirmLabel?: string;
    onConfirm: () => void | Promise<void>;
    onClose: () => void;
  }

  let { open, title, description, confirmLabel = "Delete", onConfirm, onClose }: Props = $props();

  let isDeleting = $state(false);
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open === true) {
      isDeleting = false;
      errorMessage = null;
    }
  });

  async function handleConfirm() {
    errorMessage = null;
    isDeleting = true;
    try {
      await onConfirm();
      onClose();
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Deletion failed.";
    } finally {
      isDeleting = false;
    }
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
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description>{description}</Dialog.Description>
    </Dialog.Header>

    {#if errorMessage !== null && errorMessage !== ""}
      <div
        class="border-destructive/30 bg-destructive/10 text-destructive mt-2 rounded-lg border p-3 text-xs font-medium"
        role="alert"
      >
        {errorMessage}
      </div>
    {/if}

    <Dialog.Footer class="mt-4 flex flex-row justify-end gap-2">
      <Button variant="outline" size="sm" onclick={onClose} disabled={isDeleting}>Cancel</Button>
      <Button variant="destructive" size="sm" onclick={handleConfirm} disabled={isDeleting}>
        {#if isDeleting === true}
          <span
            class="mr-2 inline-block size-3.5 animate-spin rounded-full border-2 border-white/20 border-t-white"
          ></span>
          Deleting...
        {:else}
          {confirmLabel}
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
