<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import type { Account, CreditCardAccount } from "../types";

  interface Props {
    open: boolean;
    account: Account | null;
    onClose: () => void;
  }

  let { open, account, onClose }: Props = $props();

  let bankName = $state("");
  let cardName = $state("");
  let last4 = $state("");
  let creditLimit = $state<number | string>("");
  let errorMessage = $state<string | null>(null);

  $effect(() => {
    if (open && account) {
      bankName = account.bankName;
      last4 = account.last4;
      errorMessage = null;
      if (account.type === "credit_card") {
        const cc = account as CreditCardAccount;
        cardName = cc.cardName;
        creditLimit = cc.creditLimit;
      } else {
        cardName = "";
        creditLimit = "";
      }
    }
  });

  function handleSave() {
    if (!account) return;
    errorMessage = null;

    const trimmedBank = bankName.trim();
    if (!trimmedBank) {
      errorMessage = "Bank name is required.";
      return;
    }

    const trimmedLast4 = last4.trim().replace(/\D/g, "");
    if (trimmedLast4.length !== 4) {
      errorMessage = "Last 4 digits identifier must be exactly 4 digits.";
      return;
    }

    if (account.type === "bank_account") {
      familyStore.updateBankAccount({
        id: account.id,
        bankName: trimmedBank,
        last4: trimmedLast4,
      });
    } else {
      const trimmedCard = cardName.trim();
      if (!trimmedCard) {
        errorMessage = "Card name is required.";
        return;
      }
      const limitNum = Number(creditLimit);
      if (isNaN(limitNum) || limitNum < 0) {
        errorMessage = "Credit limit must be a valid non-negative number.";
        return;
      }
      familyStore.updateCreditCard({
        id: account.id,
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
      <Dialog.Title>
        Edit {account?.type === "credit_card" ? "Credit Card" : "Bank Account"}
      </Dialog.Title>
      <Dialog.Description>
        Update account identification and institution details.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-4 py-2">
      <!-- Bank Name -->
      <div class="flex flex-col gap-1.5">
        <label for="edit-bank-name" class="text-muted-foreground text-xs font-semibold">
          Bank
        </label>
        <Input
          id="edit-bank-name"
          bind:value={bankName}
          placeholder="e.g. Chase, Wells Fargo, American Express"
        />
      </div>

      <!-- Card Name (Credit Card Only) -->
      {#if account?.type === "credit_card"}
        <div class="flex flex-col gap-1.5">
          <label for="edit-card-name" class="text-muted-foreground text-xs font-semibold">
            Card Name
          </label>
          <Input
            id="edit-card-name"
            bind:value={cardName}
            placeholder="e.g. Sapphire Preferred, Gold Card"
          />
        </div>
      {/if}

      <!-- Last 4 Digits & Credit Limit Grid -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1.5">
          <label for="edit-last4" class="text-muted-foreground text-xs font-semibold">
            Last 4 Digits
          </label>
          <Input id="edit-last4" bind:value={last4} maxlength={4} placeholder="4821" />
        </div>

        {#if account?.type === "credit_card"}
          <div class="flex flex-col gap-1.5">
            <label for="edit-limit" class="text-muted-foreground text-xs font-semibold">
              Credit Limit ($)
            </label>
            <Input
              id="edit-limit"
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
        Save Changes
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
