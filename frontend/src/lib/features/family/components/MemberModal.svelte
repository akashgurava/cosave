<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import type { Member } from "../types";

  interface Props {
    open: boolean;
    member?: Member | null | undefined;
    onClose: () => void;
  }

  let { open = $bindable(false), member = null, onClose }: Props = $props();

  let memberName = $state("");
  let errorMessage = $state<string | null>(null);

  let isEdit = $derived(member !== null && member !== undefined);
  let title = $derived(isEdit ? "Edit Member" : "Add Member");
  let description = $derived(
    isEdit
      ? "Update this family member's display name."
      : "Add an individual member to your family.",
  );
  let submitLabel = $derived(isEdit ? "Save Changes" : "Add Member");

  $effect(() => {
    if (open) {
      memberName = member !== null && member !== undefined ? member.memberName : "";
      errorMessage = null;
    }
  });

  async function handleSubmit(e?: Event) {
    if (e !== undefined) {
      e.preventDefault();
      e.stopPropagation();
    }
    errorMessage = null;
    try {
      if (isEdit && member !== null && member !== undefined) {
        await familyStore.updateMember(member.id, memberName);
      } else {
        await familyStore.addMember(memberName);
      }
      open = false;
      onClose();
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Failed to save member.";
    }
  }
</script>

<Dialog.Root
  bind:open
  onOpenChange={(isOpen) => {
    if (!isOpen) onClose();
  }}
>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description class="sr-only">{description}</Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1.5 py-2">
        <label for="member-name-input" class="text-muted-foreground text-xs font-semibold">
          Display Name
        </label>
        <Input
          id="member-name-input"
          bind:value={memberName}
          placeholder="e.g. Alex, Jordan, Morgan"
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              e.stopPropagation();
              handleSubmit(e);
            }
          }}
        />
        {#if errorMessage !== null && errorMessage !== ""}
          <p class="text-destructive text-xs font-medium">{errorMessage}</p>
        {/if}
      </div>

      <Dialog.Footer>
        <Button type="button" variant="outline" size="sm" onclick={onClose}>Cancel</Button>
        <Button
          type="submit"
          size="sm"
          class="bg-foreground text-background hover:bg-foreground/90 font-medium"
        >
          {submitLabel}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
