<script lang="ts">
  import {
    CheckIcon,
    Trash2Icon,
    CircleCheckIcon,
    ClockIcon,
    FileTextIcon,
    SaveIcon,
    CopyIcon,
    XIcon,
  } from "@lucide/svelte";
  import * as Popover from "$lib/components/ui/popover";
  import type { Transaction } from "../../types";
  import type { TransactionId } from "$lib/types";

  interface Props {
    txId: TransactionId;
    effectiveTx: Transaction;
    isDirty: boolean;
    onDraftChange: (id: TransactionId, updates: Partial<Transaction>) => void;
    onSave: (id: TransactionId) => void;
    onDiscard: (id: TransactionId) => void;
    onDelete: (id: TransactionId) => void;
  }

  let {
    txId,
    effectiveTx,
    isDirty,
    onDraftChange,
    onSave,
    onDiscard,
    onDelete,
  }: Props = $props();

  let activeDescription = $state(false);
  let copied = $state(false);
  let copyTimeoutId: ReturnType<typeof setTimeout> | null = null;

  async function copyDescription(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      if (copyTimeoutId) clearTimeout(copyTimeoutId);
      copyTimeoutId = setTimeout(() => {
        copied = false;
      }, 1500);
    } catch {
      // Fallback for restricted clipboard
    }
  }

  let confirmingDelete = $state(false);
  let deleteConfirmTimeoutId: ReturnType<typeof setTimeout> | null = null;

  function handleDeleteClick() {
    activeDescription = false;
    if (confirmingDelete) {
      if (deleteConfirmTimeoutId) clearTimeout(deleteConfirmTimeoutId);
      confirmingDelete = false;
      onDelete(txId);
    } else {
      confirmingDelete = true;
      if (deleteConfirmTimeoutId) clearTimeout(deleteConfirmTimeoutId);
      deleteConfirmTimeoutId = setTimeout(() => {
        confirmingDelete = false;
      }, 3500);
    }
  }
</script>

<svelte:window
  onclick={(e) => {
    if (confirmingDelete) {
      const path = e.composedPath();
      const isThisDeleteBtn = path.some(
        (el) => el instanceof HTMLElement && el.getAttribute("data-delete-btn") === String(txId),
      );
      if (!isThisDeleteBtn) {
        confirmingDelete = false;
      }
    }
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      confirmingDelete = false;
      activeDescription = false;
    }
  }}
/>

<div class="flex items-center justify-end gap-0.5">
  <!-- 1. Status Icon -->
  <button
    type="button"
    onclick={() =>
      onDraftChange(txId, {
        status: effectiveTx.status === "cleared" ? "pending" : "cleared",
      })}
    class="hover:bg-muted/60 inline-flex size-6 items-center justify-center rounded transition-colors"
    title="Status: {effectiveTx.status === 'cleared' ? 'Cleared' : 'Pending'} (click to toggle)"
  >
    {#if effectiveTx.status === "cleared"}
      <CircleCheckIcon class="size-3.5 text-emerald-500" />
    {:else}
      <ClockIcon class="size-3.5 text-amber-500" />
    {/if}
  </button>

  <!-- 2. Description Icon -->
  {#if effectiveTx.description && effectiveTx.description.trim().length > 0}
    <Popover.Root
      open={activeDescription}
      onOpenChange={(open) => {
        confirmingDelete = false;
        activeDescription = open;
      }}
    >
      <Popover.Trigger
        data-description-btn
        onpointerdown={(e) => {
          if (activeDescription && effectiveTx.description !== null) {
            e.preventDefault();
            copyDescription(effectiveTx.description);
          }
        }}
        class="hover:bg-muted inline-flex size-6 items-center justify-center rounded transition-colors {activeDescription
          ? 'bg-muted/80 text-foreground'
          : 'text-muted-foreground/70 hover:text-foreground'}"
        title={effectiveTx.description ?? undefined}
      >
        {#if copied}
          <CheckIcon class="size-3.5 text-emerald-500" />
        {:else if activeDescription}
          <CopyIcon class="text-foreground size-3.5" />
        {:else}
          <FileTextIcon class="size-3.5" />
        {/if}
      </Popover.Trigger>
      <Popover.Content
        side="top"
        align="center"
        sideOffset={6}
        class="border-border/60 bg-popover text-popover-foreground z-50 w-auto max-w-sm rounded-md border px-2.5 py-1 font-mono text-[11px] shadow-md select-all"
      >
        <span class="truncate">{effectiveTx.description}</span>
      </Popover.Content>
    </Popover.Root>
  {:else}
    <button
      type="button"
      disabled
      class="text-muted-foreground/20 inline-flex size-6 cursor-not-allowed items-center justify-center rounded"
      title="No statement description"
    >
      <FileTextIcon class="size-3.5" />
    </button>
  {/if}

  <!-- 3. Save Floppy Icon -->
  <button
    type="button"
    disabled={!isDirty}
    onclick={() => onSave(txId)}
    class="inline-flex size-6 items-center justify-center rounded transition-colors {isDirty
      ? 'cursor-pointer text-emerald-500 hover:bg-emerald-500/15'
      : 'text-muted-foreground/20 cursor-not-allowed'}"
    title={isDirty ? "Save changes" : "No unsaved changes"}
  >
    <SaveIcon class="size-3.5" />
  </button>

  <!-- 4. Discard X Icon -->
  <button
    type="button"
    disabled={!isDirty}
    onclick={() => onDiscard(txId)}
    class="inline-flex size-6 items-center justify-center rounded transition-colors {isDirty
      ? 'hover:bg-muted text-muted-foreground hover:text-foreground cursor-pointer'
      : 'text-muted-foreground/20 cursor-not-allowed'}"
    title={isDirty ? "Discard changes" : "No unsaved changes"}
  >
    <XIcon class="size-3.5" />
  </button>

  <!-- 5. Delete Icon -->
  {#if confirmingDelete}
    <button
      type="button"
      data-delete-btn={txId}
      onclick={(e) => {
        e.stopPropagation();
        handleDeleteClick();
      }}
      class="inline-flex size-6 items-center justify-center rounded bg-rose-500/15 text-rose-500 transition-colors hover:bg-rose-500/25"
      title="Click again to confirm delete"
    >
      <CheckIcon class="size-3.5 text-rose-500" />
    </button>
  {:else}
    <button
      type="button"
      data-delete-btn={txId}
      onclick={(e) => {
        e.stopPropagation();
        handleDeleteClick();
      }}
      class="text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 inline-flex size-6 items-center justify-center rounded transition-colors"
      title="Delete transaction (click twice to confirm)"
    >
      <Trash2Icon class="size-3.5" />
    </button>
  {/if}
</div>
