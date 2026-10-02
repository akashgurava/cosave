<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import type { Account, AccountType, CurrencyCode } from "../types";

  interface Props {
    open: boolean;
    account?: Account | null | undefined;
    defaultMemberId?: number | undefined;
    defaultType?: AccountType | undefined;
    onClose: () => void;
  }

  let {
    open = $bindable(false),
    account = null,
    defaultMemberId,
    defaultType = "bank_account",
    onClose,
  }: Props = $props();

  let selectedOwnerId = $state("");
  let selectedCurrency = $state<CurrencyCode>(familyStore.currency);
  let accountType = $state<AccountType>("bank_account");
  let bankName = $state("");
  let accountName = $state("");
  let cardName = $state("");
  let last4 = $state("");
  let availableBalance = $state<number | string>("");
  let creditLimit = $state<number | string>("");
  let availableCredit = $state<number | string>("");
  let errorMessage = $state<string | null>(null);

  let isEdit = $derived(account !== null && account !== undefined);
  let modalTitle = $derived(
    isEdit
      ? `Edit ${account?.type === "credit_card" ? "Credit Card" : "Bank Account"}`
      : "Add Account",
  );
  let modalDescription = $derived(
    isEdit
      ? "Update account identification, institution, and balance details."
      : "Add a financial account for a member.",
  );
  let submitLabel = $derived(
    isEdit
      ? "Save Changes"
      : accountType === "bank_account"
        ? "Add Bank Account"
        : "Add Credit Card",
  );

  let calculatedOutstanding = $derived.by(() => {
    const limit = Number(creditLimit);
    const avail = Number(availableCredit);
    if (!isNaN(limit) && !isNaN(avail)) {
      return Math.max(0, limit - avail).toFixed(2);
    }
    return "0.00";
  });

  $effect(() => {
    if (open) {
      errorMessage = null;
      if (account) {
        selectedOwnerId = String(account.owner_member_id);
        selectedCurrency =
          familyStore.getCurrencyOption(account.currency_id)?.code ?? familyStore.currency;
        accountType = account.type;
        bankName = account.bank_name;
        last4 = account.last4;
        if (account.type === "bank_account") {
          accountName = account.account_name;
          availableBalance = (account.available_balance_cents / 100).toFixed(2);
          cardName = "";
          creditLimit = "";
          availableCredit = "";
        } else {
          accountName = "";
          availableBalance = "";
          cardName = account.card_name;
          creditLimit = (account.credit_limit_cents / 100).toFixed(2);
          availableCredit = (account.available_cents / 100).toFixed(2);
        }
      } else {
        const firstMember = familyStore.members[0];
        selectedOwnerId =
          defaultMemberId !== undefined
            ? String(defaultMemberId)
            : firstMember !== undefined
              ? String(firstMember.id)
              : "";
        selectedCurrency = familyStore.currency;
        accountType = defaultType;
        bankName = "";
        accountName = "";
        cardName = "";
        last4 = "";
        availableBalance = "";
        creditLimit = "";
        availableCredit = "";
      }
    }
  });

  async function handleSave() {
    errorMessage = null;

    if (!selectedOwnerId) {
      errorMessage = "Please select an account owner.";
      return;
    }
    const ownerId = Number(selectedOwnerId);

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

    try {
      if (accountType === "bank_account") {
        const trimmedAccountName = accountName.trim();
        if (!trimmedAccountName) {
          errorMessage = "Account name is required (e.g. Primary Checking, Emergency Savings).";
          return;
        }
        const balanceNum = availableBalance === "" ? 0 : Number(availableBalance);
        if (isNaN(balanceNum)) {
          errorMessage = "Please provide a valid numeric available balance.";
          return;
        }

        const isDuplicate = familyStore.accounts.some(
          (a) =>
            a.owner_member_id === ownerId &&
            (a.type === "bank_account" ? a.account_name : a.card_name).toLowerCase() ===
              trimmedAccountName.toLowerCase() &&
            (!isEdit || a.id !== account?.id),
        );
        if (isDuplicate) {
          errorMessage = "An account with this name already exists for this member.";
          return;
        }

        if (isEdit && account) {
          await familyStore.updateBankAccount(account.id, {
            currency: selectedCurrency,
            bank_name: trimmedBank,
            account_name: trimmedAccountName,
            last4: trimmedLast4,
            available_balance_cents: Math.round(balanceNum * 100),
          });
        } else {
          await familyStore.addBankAccount({
            owner_member_id: ownerId,
            currency: selectedCurrency,
            bank_name: trimmedBank,
            account_name: trimmedAccountName,
            last4: trimmedLast4,
            available_balance_cents: Math.round(balanceNum * 100),
          });
        }
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

        const availNum = availableCredit === "" ? limitNum : Number(availableCredit);
        if (isNaN(availNum) || availNum < 0) {
          errorMessage = "Please provide a valid non-negative available credit amount.";
          return;
        }

        const isDuplicate = familyStore.accounts.some(
          (a) =>
            a.owner_member_id === ownerId &&
            (a.type === "bank_account" ? a.account_name : a.card_name).toLowerCase() ===
              trimmedCard.toLowerCase() &&
            (!isEdit || a.id !== account?.id),
        );
        if (isDuplicate) {
          errorMessage = "An account with this name already exists for this member.";
          return;
        }

        if (isEdit && account) {
          await familyStore.updateCreditCard(account.id, {
            currency: selectedCurrency,
            bank_name: trimmedBank,
            card_name: trimmedCard,
            last4: trimmedLast4,
            credit_limit_cents: Math.round(limitNum * 100),
            available_cents: Math.round(availNum * 100),
          });
        } else {
          await familyStore.addCreditCard({
            owner_member_id: ownerId,
            currency: selectedCurrency,
            bank_name: trimmedBank,
            card_name: trimmedCard,
            last4: trimmedLast4,
            credit_limit_cents: Math.round(limitNum * 100),
            available_cents: Math.round(availNum * 100),
          });
        }
      }

      open = false;
      onClose();
    } catch (err: unknown) {
      errorMessage = err instanceof Error ? err.message : "Failed to save account.";
    }
  }
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => !isOpen && onClose()}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{modalTitle}</Dialog.Title>
      <Dialog.Description class="sr-only">{modalDescription}</Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSave} class="flex flex-col gap-4 py-2">
      <!-- Account Type Segmented Toggle (Add mode only) -->
      {#if !isEdit}
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
              class="rounded-md py-1.5 text-xs font-semibold transition-all {accountType ===
              'bank_account'
                ? 'bg-background text-foreground shadow-xs'
                : 'text-muted-foreground hover:text-foreground'}"
              onclick={() => (accountType = "bank_account")}
            >
              Bank Account
            </button>
            <button
              type="button"
              role="radio"
              aria-checked={accountType === "credit_card"}
              class="rounded-md py-1.5 text-xs font-semibold transition-all {accountType ===
              'credit_card'
                ? 'bg-background text-foreground shadow-xs'
                : 'text-muted-foreground hover:text-foreground'}"
              onclick={() => (accountType = "credit_card")}
            >
              Credit Card
            </button>
          </div>
        </div>

        <!-- Account Owner Selector (Add mode only) -->
        <div class="flex flex-col gap-1.5">
          <label for="owner-select" class="text-muted-foreground text-xs font-semibold">
            Account Owner
          </label>
          <Select.Root bind:value={selectedOwnerId} type="single">
            <Select.Trigger id="owner-select" class="w-full">
              <span>
                {familyStore.members.find((m) => String(m.id) === selectedOwnerId)?.member_name ??
                  "Select a member..."}
              </span>
            </Select.Trigger>
            <Select.Content>
              {#each familyStore.members as member (member.id)}
                <Select.Item value={String(member.id)} label={member.member_name}>
                  {member.member_name}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      {/if}

      <!-- Bank Name and Currency Grid -->
      <div class="grid grid-cols-2 gap-3">
        <div class="flex flex-col gap-1.5">
          <label for="bank-name-input" class="text-muted-foreground text-xs font-semibold">
            Bank Name
          </label>
          <Input id="bank-name-input" bind:value={bankName} placeholder="e.g. Chase, Ally, Amex" />
        </div>

        <div class="flex flex-col gap-1.5">
          <label for="currency-select" class="text-muted-foreground text-xs font-semibold">
            Currency
          </label>
          <Select.Root bind:value={selectedCurrency} type="single">
            <Select.Trigger id="currency-select" class="w-full">
              <span class="font-mono font-semibold"
                >{familyStore.getCurrencySymbol(selectedCurrency)}</span
              >
              <span>{selectedCurrency}</span>
            </Select.Trigger>
            <Select.Content class="max-h-56">
              {#each familyStore.currencies as curr (curr.code)}
                <Select.Item value={curr.code} label={`${curr.symbol} ${curr.code} - ${curr.name}`}>
                  <div class="flex items-center gap-2 text-xs">
                    <span class="text-muted-foreground w-5 text-center font-mono font-bold">
                      {curr.symbol}
                    </span>
                    <span class="font-semibold">{curr.code}</span>
                    <span class="text-muted-foreground text-[11px]">&bull; {curr.name}</span>
                  </div>
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </div>

      <!-- Account Name (Bank Account) -->
      {#if accountType === "bank_account"}
        <div class="flex flex-col gap-1.5">
          <label for="account-name-input" class="text-muted-foreground text-xs font-semibold">
            Account Name
          </label>
          <Input
            id="account-name-input"
            bind:value={accountName}
            placeholder="e.g. Total Checking, Emergency Savings"
          />
        </div>
      {/if}

      <!-- Card Name (Credit Card) -->
      {#if accountType === "credit_card"}
        <div class="flex flex-col gap-1.5">
          <label for="card-name-input" class="text-muted-foreground text-xs font-semibold">
            Card Name
          </label>
          <Input
            id="card-name-input"
            bind:value={cardName}
            placeholder="e.g. Sapphire Preferred, Gold Card"
          />
        </div>
      {/if}

      <!-- Last 4 Digits & Balance (Bank Account) -->
      {#if accountType === "bank_account"}
        <div class="grid grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label for="last4-input" class="text-muted-foreground text-xs font-semibold">
              Last 4 Digits
            </label>
            <Input id="last4-input" bind:value={last4} maxlength={4} placeholder="4821" />
          </div>

          <div class="flex flex-col gap-1.5">
            <label for="balance-input" class="text-muted-foreground text-xs font-semibold">
              Available ({familyStore.getCurrencySymbol(selectedCurrency)})
            </label>
            <Input
              id="balance-input"
              type="number"
              step="0.01"
              bind:value={availableBalance}
              placeholder="0.00"
            />
          </div>
        </div>
      {/if}

      <!-- Last 4 Digits & Credit Limit (Credit Card) -->
      {#if accountType === "credit_card"}
        <div class="grid grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label for="card-last4-input" class="text-muted-foreground text-xs font-semibold">
              Last 4 Digits
            </label>
            <Input id="card-last4-input" bind:value={last4} maxlength={4} placeholder="5561" />
          </div>

          <div class="flex flex-col gap-1.5">
            <label for="credit-limit-input" class="text-muted-foreground text-xs font-semibold">
              Credit Limit ({familyStore.getCurrencySymbol(selectedCurrency)})
            </label>
            <Input
              id="credit-limit-input"
              type="number"
              min="0"
              step="100"
              bind:value={creditLimit}
              placeholder="20000.00"
            />
          </div>
        </div>

        <!-- Available Credit & Calculated Outstanding (Credit Card) -->
        <div class="grid grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label for="available-credit-input" class="text-muted-foreground text-xs font-semibold">
              Available Credit ({familyStore.getCurrencySymbol(selectedCurrency)})
            </label>
            <Input
              id="available-credit-input"
              type="number"
              min="0"
              step="100"
              bind:value={availableCredit}
              placeholder="17850.00"
            />
          </div>

          <div class="flex flex-col gap-1.5">
            <label for="outstanding-preview" class="text-muted-foreground text-xs font-semibold">
              Outstanding ({familyStore.getCurrencySymbol(selectedCurrency)})
            </label>
            <div
              id="outstanding-preview"
              class="border-border/40 bg-muted/20 flex h-9 items-center rounded-md border px-3 text-sm font-semibold"
            >
              {familyStore.getCurrencySymbol(selectedCurrency)}{calculatedOutstanding}
            </div>
          </div>
        </div>
      {/if}

      {#if errorMessage}
        <p class="text-destructive text-xs font-medium">{errorMessage}</p>
      {/if}

      <Dialog.Footer class="pt-2">
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
