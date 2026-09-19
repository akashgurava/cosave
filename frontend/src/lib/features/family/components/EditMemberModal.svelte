<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import type { Member } from "../types";

  interface Props {
    open: boolean;
    member: Member | null;
    onClose: () => void;
  }

  let { open, member, onClose }: Props = $props();

  let memberName = $state("");
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open && member) {
      memberName = member.name;
      errorMessage = null;
    }
  });

  function handleSave() {
    if (!member) return;
    const trimmed = memberName.trim();
    if (!trimmed) {
      errorMessage = "Member name cannot be empty.";
      return;
    }
    familyStore.updateMember({ id: member.id, name: trimmed });
    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Edit Member</Dialog.Title>
      <Dialog.Description>Update this family member's display name.</Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-4 py-2">
      <div class="flex flex-col gap-1.5">
        <label for="edit-member-name-input" class="text-muted-foreground text-xs font-semibold">
          Display Name
        </label>
        <Input
          id="edit-member-name-input"
          bind:value={memberName}
          placeholder="e.g. Alex"
          onkeydown={(e) => e.key === "Enter" && handleSave()}
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
        onclick={handleSave}
      >
        Save Changes
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
