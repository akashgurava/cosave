<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let memberName = $state("");
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open) {
      memberName = "";
      errorMessage = null;
    }
  });

  function handleCreate() {
    const trimmed = memberName.trim();
    if (!trimmed) {
      errorMessage = "Member name is required.";
      return;
    }
    familyStore.addMember({ name: trimmed });
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Add Family Member</Dialog.Title>
      <Dialog.Description>Add an individual member to your household family.</Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-4 py-2">
      <div class="flex flex-col gap-1.5">
        <label for="member-name-input" class="text-muted-foreground text-xs font-semibold">
          Display Name
        </label>
        <Input
          id="member-name-input"
          bind:value={memberName}
          placeholder="e.g. Alex, Jordan, Morgan"
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
        />
        {#if errorMessage}
          <p class="text-destructive text-xs font-medium">{errorMessage}</p>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button
        size="sm"
        class="bg-foreground text-background hover:bg-foreground/90 font-medium"
        onclick={handleCreate}
      >
        Add Member
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
