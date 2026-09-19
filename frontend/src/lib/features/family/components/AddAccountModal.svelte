<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import type { AccountType } from "../types";

  interface Props {
    open: boolean;
    defaultMemberId?: string;
    defaultType?: AccountType;
    onClose: () => void;
  }

  let { open, defaultMemberId, defaultType = "bank_account", onClose }: Props = $props();

  let selectedOwnerId = $state("");
  let accountType = $state<AccountType>("bank_account");
  let bankName = $state("");
  let cardName = $state("");
  let last4 = $state("");
  let creditLimit = $state<number | string>("");
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open) {
      selectedOwnerId = defaultMemberId || familyStore.members[0]?.id || "";
      accountType = defaultType;
      bankName = "";
      cardName = "";
      last4 = "";
      creditLimit = "";
      errorMessage = null;
    }
  });

  function handleSave() {
    errorMessage = null;

    if (!selectedOwnerId) {
      errorMessage = "Please select an account owner.";
      return;
    }

    const trimmedBank = bankName.trim();
    if (!trimmedBank) {
      errorMessage = "Bank name is required.";
      return;
    }

    const trimmedLast4 = last4.trim().replace(/\D/g, "");
    if (trimmedLast4.length !== 4) {
      errorMessage = "Please enter exactly 4 digits for identification.";
      return;
    }

    if (accountType === "bank_account") {
      familyStore.addBankAccount({
        ownerMemberId: selectedOwnerId,
        bankName: trimmedBank,
        last4: trimmedLast4,
      });
    } else {
      const trimmedCard = cardName.trim();
      if (!trimmedCard) {
        errorMessage = "Card name is required (e.g. Gold Card, Double Cash).";
        return;
      }
      const limitNum = Number(creditLimit);
      if (isNaN(limitNum) || limitNum < 0) {
        errorMessage = "Please provide a valid non-negative credit limit.";
        return;
      }

      familyStore.addCreditCard({
        ownerMemberId: selectedOwnerId,
        bankName: trimmedBank,
        cardName: trimmedCard,
        last4: trimmedLast4,
        creditLimit: limitNum,
      });
    }

    onClose();
  }
</script>

<Dialog.Root {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Add Account</Dialog.Title>
      <Dialog.Description class="sr-only">Link a financial account to a member.</Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-4 py-2">
      <!-- Account Type Segmented Toggle -->
      <div class="flex flex-col gap-1.5">
        <span id="account-type-label" class="text-muted-foreground text-xs font-semibold">
          Account Type
        </span>
        <div
          role="radiogroup"
          aria-labelledby="account-type-label"
          class="border-border/40 bg-muted/30 grid grid-cols-2 gap-1 rounded-lg border p-1"
        >
          <button
            type="button"
            role="radio"
            aria-checked={accountType === "bank_account"}
            class={`rounded-md py-1.5 text-xs font-medium transition-all ${
              accountType === "bank_account"
                ? "bg-foreground text-background font-semibold shadow-xs"
                : "text-muted-foreground hover:text-foreground"
            }`}
            onclick={() => (accountType = "bank_account")}
          >
            Bank Account
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={accountType === "credit_card"}
            class={`rounded-md py-1.5 text-xs font-medium transition-all ${
              accountType === "credit_card"
                ? "bg-foreground text-background font-semibold shadow-xs"
                : "text-muted-foreground hover:text-foreground"
            }`}
            onclick={() => (accountType = "credit_card")}
          >
            Credit Card
          </button>
        </div>
      </div>

      <!-- Owner Member Select -->
      <div class="flex flex-col gap-1.5">
        <label for="owner-select" class="text-muted-foreground text-xs font-semibold">Owner</label>
        <Select.Root bind:value={selectedOwnerId} type="single">
          <Select.Trigger id="owner-select" class="w-full">
            {familyStore.members.find((m) => m.id === selectedOwnerId)?.name ?? "Select member"}
          </Select.Trigger>
          <Select.Content>
            {#each familyStore.members as member (member.id)}
              <Select.Item value={member.id} label={member.name}>{member.name}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <!-- Bank Name -->
      <div class="flex flex-col gap-1.5">
        <label for="bank-name-input" class="text-muted-foreground text-xs font-semibold">
          Bank
        </label>
        <Input
          id="bank-name-input"
          bind:value={bankName}
          placeholder="e.g. Chase, Wells Fargo, American Express"
        />
      </div>

      <!-- Card Name (Credit Card Only) -->
      {#if accountType === "credit_card"}
        <div class="flex flex-col gap-1.5">
          <label for="card-name-input" class="text-muted-foreground text-xs font-semibold">
            Card Name
          </label>
          <Input
            id="card-name-input"
            bind:value={cardName}
            placeholder="e.g. Sapphire Preferred, Gold Card, Apple Card"
          />
        </div>
      {/if}

      <!-- Last 4 Digits & Credit Limit in Grid -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1.5">
          <label for="last4-input" class="text-muted-foreground text-xs font-semibold">
            Last 4 Digits
          </label>
          <Input id="last4-input" bind:value={last4} maxlength={4} placeholder="4821" />
        </div>

        {#if accountType === "credit_card"}
          <div class="flex flex-col gap-1.5">
            <label for="credit-limit-input" class="text-muted-foreground text-xs font-semibold">
              Credit Limit ($)
            </label>
            <Input
              id="credit-limit-input"
              type="number"
              min="0"
              step="500"
              bind:value={creditLimit}
              placeholder="10000"
            />
          </div>
        {/if}
      </div>

      {#if errorMessage}
        <p class="text-destructive text-xs font-medium">{errorMessage}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" size="sm" onclick={onClose}>Cancel</Button>
      <Button
        size="sm"
        class="bg-foreground text-background hover:bg-foreground/90 font-medium"
        onclick={handleSave}
      >
        Add Account
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
