<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import InputField from "$lib/components/InputField.svelte";
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
  let nameField = $state<ReturnType<typeof InputField> | null>(null);

  let isEdit = $derived(member !== null && member !== undefined);
  let title = $derived(isEdit === true ? "Edit Member" : "Add Member");
  let description = $derived(
    isEdit === true
      ? "Update this family member's display name."
      : "Add an individual member to your family.",
  );
  let submitLabel = $derived(isEdit === true ? "Save Changes" : "Add Member");

  $effect(() => {
    if (open === true) {
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

    if (nameField !== null && nameField.validate() === false) {
      return;
    }

    const trimmed = memberName.trim();
    try {
      if (isEdit === true && member !== null && member !== undefined) {
        await familyStore.updateMember(member.id, trimmed);
      } else {
        await familyStore.addMember(trimmed);
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
    if (isOpen === false) onClose();
  }}
>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{title}</Dialog.Title>
      <Dialog.Description class="sr-only">{description}</Dialog.Description>
    </Dialog.Header>

    <form novalidate onsubmit={handleSubmit} class="flex flex-col gap-4">
      <InputField
        bind:this={nameField}
        id="member-name-input"
        label="Name"
        required
        placeholder="e.g. Alex, Jordan, Morgan"
        bind:value={memberName}
        error={errorMessage}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            handleSubmit(e);
          }
        }}
      />

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
