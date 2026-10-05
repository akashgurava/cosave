<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import { toAmountCents, type AmountCents } from "$lib/types/core";
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

  $effect(() => {
    if (open) {
      errorMessage = null;
      if (account !== null && account !== undefined) {
        selectedOwnerId = String(account.ownerMemberId);
        selectedCurrency =
          familyStore.getCurrencyOption(account.currencyId)?.code ?? familyStore.currency;
        accountType = account.type;
        bankName = account.bankName;
        last4 = account.last4;
        if (account.type === "bank_account") {
          accountName = account.accountName;
          availableBalance = (account.availableBalanceCents / 100).toFixed(2);
          cardName = "";
          creditLimit = "";
          availableCredit = "";
        } else {
          accountName = "";
          availableBalance = "";
          cardName = account.cardName;
          creditLimit = (account.creditLimitCents / 100).toFixed(2);
          availableCredit = (account.availableCents / 100).toFixed(2);
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

  function parseInputToCents(val: number | string): AmountCents {
    const str = String(val).trim();
    if (str === "" || str === "0") {
      return toAmountCents(0);
    }
    const num = Number(str);
    if (Number.isNaN(num)) {
      return toAmountCents(0);
    }
    return toAmountCents(Math.round(num * 100));
  }

  async function handleSave() {
    errorMessage = null;

    const ownerId = selectedOwnerId !== "" ? Number(selectedOwnerId) : 0;
    const balanceCents = parseInputToCents(availableBalance);
    const limitCents = parseInputToCents(creditLimit);
    const availCents = parseInputToCents(availableCredit);

    try {
      if (accountType === "bank_account") {
        if (isEdit && account !== null && account !== undefined) {
          await familyStore.updateBankAccount(account.id, {
            currency: selectedCurrency,
            bankName,
            accountName,
            last4,
            availableBalanceCents: balanceCents,
          });
        } else {
          await familyStore.addBankAccount({
            ownerMemberId: ownerId,
            currency: selectedCurrency,
            bankName,
            accountName,
            last4,
            availableBalanceCents: balanceCents,
          });
        }
      } else {
        if (isEdit && account !== null && account !== undefined) {
          await familyStore.updateCreditCard(account.id, {
            currency: selectedCurrency,
            bankName,
            cardName,
            last4,
            creditLimitCents: limitCents,
            availableCents: availCents,
          });
        } else {
          await familyStore.addCreditCard({
            ownerMemberId: ownerId,
            currency: selectedCurrency,
            bankName,
            cardName,
            last4,
            creditLimitCents: limitCents,
            availableCents: availCents,
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

<Dialog.Root
  bind:open
  onOpenChange={(isOpen) => {
    if (!isOpen) onClose();
  }}
>
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
                {familyStore.members.find((m) => String(m.id) === selectedOwnerId)?.memberName ??
                  "Select a member..."}
              </span>
            </Select.Trigger>
            <Select.Content>
              {#each familyStore.members as member (member.id)}
                <Select.Item value={String(member.id)} label={member.memberName}>
                  {member.memberName}
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

      <!-- Credit Card Fields -->
      {#if accountType === "credit_card"}
        <div class="flex flex-col gap-1.5">
          <label for="card-last4-input" class="text-muted-foreground text-xs font-semibold">
            Last 4 Digits
          </label>
          <Input id="card-last4-input" bind:value={last4} maxlength={4} placeholder="5561" />
        </div>

        <div class="grid grid-cols-2 gap-3">
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
        </div>
      {/if}

      {#if errorMessage !== null && errorMessage !== ""}
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
