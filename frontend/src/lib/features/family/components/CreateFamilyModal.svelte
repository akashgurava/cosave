<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import UsersIcon from "@lucide/svelte/icons/users";

  interface Props {
    open: boolean;
  }

  let { open = $bindable(false) }: Props = $props();

  let familyName = $state("");
  let memberName = $state("");
  let selectedCurrencyId = $state("");
  let errorMessage = $state<string | null>(null);
  let isSubmitting = $state(false);

  // Initialize form defaults when modal opens
  $effect(() => {
    if (open === true) {
      familyName = "";
      memberName = "";
      errorMessage = null;
      isSubmitting = false;

      // Default to family's current currencyId or first currency
      if (familyStore.currencies.length > 0) {
        const currId = familyStore.currencyId;
        if (currId !== null) {
          selectedCurrencyId = String(currId);
        } else {
          const first = familyStore.currencies[0];
          if (first !== undefined) {
            selectedCurrencyId = String(first.id);
          }
        }
      }
    }
  });

  const selectedCurrencyOption = $derived.by(() => {
    const idNum = Number(selectedCurrencyId);
    const found = familyStore.currencies.find((c) => c.id === idNum);
    if (found !== undefined) {
      return found;
    }
    const first = familyStore.currencies[0];
    if (first !== undefined) {
      return first;
    }
    return null;
  });

  async function handleSubmit(e?: Event) {
    if (e !== undefined) {
      e.preventDefault();
      e.stopPropagation();
    }
    errorMessage = null;

    const trimmedFamily = familyName.trim();
    if (trimmedFamily.length === 0) {
      errorMessage = "Please enter a family name.";
      return;
    }

    const trimmedMember = memberName.trim();
    if (trimmedMember.length === 0) {
      errorMessage = "Please enter the first family member's name.";
      return;
    }

    const currId = Number(selectedCurrencyId);
    if (Number.isInteger(currId) === false || currId <= 0) {
      errorMessage = "Please select a currency.";
      return;
    }

    isSubmitting = true;
    try {
      // 1. Create family with name and selected currency
      await familyStore.createFamily({
        familyName: trimmedFamily,
        currencyId: currId,
      });

      // 2. Add first member
      await familyStore.addMember(trimmedMember);

      // Successfully onboarded family
      open = false;
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Failed to initialize family.";
    } finally {
      isSubmitting = false;
    }
  }
</script>

<Dialog.Root
  bind:open
  onOpenChange={(isOpen) => {
    // Prevent closing if family is still empty
    if (isOpen === false && familyStore.members.length === 0) {
      open = true;
    }
  }}
>
  <Dialog.Content class="sm:max-w-md [&>button]:hidden">
    <Dialog.Header>
      <div class="flex items-center gap-2.5">
        <div
          class="bg-foreground text-background flex size-9 items-center justify-center rounded-lg shadow-xs"
        >
          <UsersIcon class="size-4" />
        </div>
        <div>
          <Dialog.Title class="text-foreground text-lg font-bold tracking-tight">
            Welcome to CoSave
          </Dialog.Title>
          <Dialog.Description class="text-muted-foreground text-xs">
            Set up your family, first member, and currency to get started.
          </Dialog.Description>
        </div>
      </div>
    </Dialog.Header>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4 pt-2">
      <!-- Family Name -->
      <div class="flex flex-col gap-1.5">
        <label for="family-name" class="text-muted-foreground text-xs font-semibold">
          Family Name
        </label>
        <Input
          id="family-name"
          bind:value={familyName}
          placeholder="e.g. The Smiths, Miller Family"
          disabled={isSubmitting}
          autofocus
        />
      </div>

      <!-- First Member Name -->
      <div class="flex flex-col gap-1.5">
        <label for="member-name" class="text-muted-foreground text-xs font-semibold">
          First Member Name
        </label>
        <Input
          id="member-name"
          bind:value={memberName}
          placeholder="e.g. Alice, Bob"
          disabled={isSubmitting}
        />
      </div>

      <!-- Currency Selector -->
      <div class="flex flex-col gap-1.5">
        <label for="currency-select" class="text-muted-foreground text-xs font-semibold">
          Currency
        </label>
        <Select.Root bind:value={selectedCurrencyId} type="single" disabled={isSubmitting}>
          <Select.Trigger id="currency-select" class="w-full">
            <span>
              {#if selectedCurrencyOption !== null}
                {selectedCurrencyOption.code} &bull; {selectedCurrencyOption.name} ({selectedCurrencyOption.symbol})
              {:else}
                Select Currency
              {/if}
            </span>
          </Select.Trigger>
          <Select.Content class="max-h-56">
            {#each familyStore.currencies as cur (cur.id)}
              <Select.Item value={String(cur.id)} label={`${cur.code} - ${cur.name}`}>
                <span class="font-mono font-medium">{cur.code}</span>
                <span class="text-muted-foreground ml-2 text-xs">{cur.name} ({cur.symbol})</span>
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      {#if errorMessage !== null}
        <div
          class="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border p-2.5 text-xs font-medium"
        >
          {errorMessage}
        </div>
      {/if}

      <Dialog.Footer class="mt-2">
        <Button
          type="submit"
          class="bg-foreground text-background hover:bg-foreground/90 w-full font-medium"
          disabled={isSubmitting}
        >
          {isSubmitting ? "Setting up family..." : "Get Started"}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
