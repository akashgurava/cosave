<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { familyStore } from "../store.svelte";
  import { formatMoneyInput, parseMoneyInput } from "../currency";
  import { toMemberId, type Account, type AccountType, type MemberId } from "../types";

  interface Props {
    open: boolean;
    account?: Account | null | undefined;
    defaultMemberId?: MemberId | undefined;
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
  let accountType = $state<AccountType>("bank_account");
  let bankName = $state("");
  let accountName = $state("");
  let cardName = $state("");
  let last4 = $state("");
  let availableBalance = $state<string>("");
  let creditLimit = $state<string>("");
  let availableCredit = $state<string>("");
  let errorMessage = $state<string | null>(null);

  const currencyCode = $derived(familyStore.currency);
  const scale = $derived(familyStore.getCurrencyScale());
  const currencySymbol = $derived(familyStore.getCurrencySymbol());
  const numberStep = $derived(scale === 0 ? "1" : scale === 3 ? "0.001" : "0.01");
  const zeroPlaceholder = $derived(scale === 0 ? "0" : scale === 3 ? "0.000" : "0.00");

  let isEdit = $derived(account !== null && account !== undefined);
  let modalTitle = $derived(
    isEdit === true && account !== null && account !== undefined
      ? `Edit ${account.type === "credit_card" ? "Credit Card" : "Bank Account"}`
      : "Add Account",
  );
  let modalDescription = $derived(
    isEdit === true
      ? "Update account identification, institution, and balance details."
      : "Add a financial account for a member.",
  );
  let submitLabel = $derived(
    isEdit === true
      ? "Save Changes"
      : accountType === "bank_account"
        ? "Add Bank Account"
        : "Add Credit Card",
  );
  let selectedMemberName = $derived.by(() => {
    if (selectedOwnerId === "") return "Select a member...";
    const idNum = Number(selectedOwnerId);
    if (Number.isInteger(idNum) === false || idNum <= 0) return "Select a member...";
    const member = familyStore.getMember(toMemberId(idNum));
    return member !== null ? member.memberName : "Select a member...";
  });

  $effect(() => {
    if (open === true) {
      errorMessage = null;
      if (account !== null && account !== undefined) {
        selectedOwnerId = String(account.ownerMemberId);
        accountType = account.type;
        bankName = account.bankName;
        last4 = account.last4;
        if (account.type === "bank_account") {
          accountName = account.accountName;
          availableBalance = formatMoneyInput(account.availableBalance, scale);
          cardName = "";
          creditLimit = "";
          availableCredit = "";
        } else {
          accountName = "";
          availableBalance = "";
          cardName = account.cardName;
          creditLimit = formatMoneyInput(account.creditLimit, scale);
          availableCredit = formatMoneyInput(account.availableCredit, scale);
        }
      } else {
        const firstMember = familyStore.members[0];
        selectedOwnerId =
          defaultMemberId !== undefined
            ? String(defaultMemberId)
            : firstMember !== undefined
              ? String(firstMember.id)
              : "";
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

    const idNum = Number(selectedOwnerId);
    if (selectedOwnerId === "" || Number.isInteger(idNum) === false || idNum <= 0) {
      errorMessage = "Please select a member.";
      return;
    }
    const ownerId = toMemberId(idNum);
    const balanceUnits = parseMoneyInput(availableBalance, scale);
    const limitUnits = parseMoneyInput(creditLimit, scale);
    const availUnits = parseMoneyInput(availableCredit, scale);

    try {
      if (accountType === "bank_account") {
        if (isEdit === true && account !== null && account !== undefined) {
          await familyStore.updateBankAccount(account.id, {
            bankName,
            accountName,
            last4,
            availableBalance: balanceUnits,
          });
        } else {
          await familyStore.addBankAccount({
            ownerMemberId: ownerId,
            bankName,
            accountName,
            last4,
            availableBalance: balanceUnits,
          });
        }
      } else {
        if (isEdit === true && account !== null && account !== undefined) {
          await familyStore.updateCreditCard(account.id, {
            bankName,
            cardName,
            last4,
            creditLimit: limitUnits,
            availableCredit: availUnits,
          });
        } else {
          await familyStore.addCreditCard({
            ownerMemberId: ownerId,
            bankName,
            cardName,
            last4,
            creditLimit: limitUnits,
            availableCredit: availUnits,
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
    if (isOpen === false) onClose();
  }}
>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>{modalTitle}</Dialog.Title>
      <Dialog.Description class="sr-only">{modalDescription}</Dialog.Description>
    </Dialog.Header>

    <form onsubmit={handleSave} class="flex flex-col gap-4 py-2">
      <!-- Account Type Segmented Toggle (Add mode only) -->
      {#if isEdit === false}
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
              <span>{selectedMemberName}</span>
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

      <!-- Bank Name and Family Currency Indicator -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center justify-between">
          <label for="bank-name-input" class="text-muted-foreground text-xs font-semibold">
            Bank Name
          </label>
          <span class="text-muted-foreground/80 flex items-center gap-1 text-[11px]">
            Family Currency:
            <span class="text-foreground font-mono font-semibold"
              >{currencyCode} ({currencySymbol})</span
            >
          </span>
        </div>
        <Input id="bank-name-input" bind:value={bankName} placeholder="e.g. Chase, Ally, Amex" />
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
              Available ({currencySymbol})
            </label>
            <Input
              id="balance-input"
              type="number"
              step={numberStep}
              bind:value={availableBalance}
              placeholder={zeroPlaceholder}
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
              Credit Limit ({currencySymbol})
            </label>
            <Input
              id="credit-limit-input"
              type="number"
              min="0"
              step={numberStep}
              bind:value={creditLimit}
              placeholder={zeroPlaceholder}
            />
          </div>

          <div class="flex flex-col gap-1.5">
            <label for="available-credit-input" class="text-muted-foreground text-xs font-semibold">
              Available Credit ({currencySymbol})
            </label>
            <Input
              id="available-credit-input"
              type="number"
              min="0"
              step={numberStep}
              bind:value={availableCredit}
              placeholder={zeroPlaceholder}
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
