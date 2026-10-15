<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Button } from "$lib/components/ui/button";
  import InputField from "$lib/components/InputField.svelte";
  import { familyStore } from "../store.svelte";
  import { formatMoneyInput, parseMoneyInput } from "../currency";
  import { MemberId, type Account, type AccountType } from "../types";

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
  let name = $state("");
  let availableBalance = $state<string>("");
  let creditLimit = $state<string>("");
  let availableCredit = $state<string>("");
  let errorMessage = $state<string | null>(null);

  const scale = $derived(familyStore.getCurrencyScale());
  const currencySymbol = $derived(familyStore.getCurrencySymbol());
  const zeroPlaceholder = $derived(scale === 0 ? "0" : scale === 3 ? "0.000" : "0.00");

  let isEdit = $derived(account !== null && account !== undefined);
  let modalTitle = $derived(
    isEdit === true && account !== null && account !== undefined
      ? `Edit ${account.type === "credit_card" ? "Credit Card" : "Bank Account"}`
      : "Add Account",
  );
  let modalDescription = $derived(
    isEdit === true
      ? "Update account name and balance details."
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
    const member = familyStore.getMember(MemberId(idNum));
    return member !== null ? member.memberName : "Select a member...";
  });

  $effect(() => {
    if (open === true) {
      errorMessage = null;
      if (account !== null && account !== undefined) {
        selectedOwnerId = String(account.ownerMemberId);
        accountType = account.type;
        name = account.name;
        if (account.type === "bank_account") {
          availableBalance = formatMoneyInput(account.availableBalance, scale);
          creditLimit = "";
          availableCredit = "";
        } else {
          availableBalance = "";
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
        name = "";
        availableBalance = "";
        creditLimit = "";
        availableCredit = "";
      }
    }
  });

  function handleAmountKeyDown(e: KeyboardEvent, currentVal: string, allowNegative = false) {
    if (
      e.key === "Backspace" ||
      e.key === "Delete" ||
      e.key === "Tab" ||
      e.key === "ArrowLeft" ||
      e.key === "ArrowRight" ||
      e.key === "ArrowUp" ||
      e.key === "ArrowDown" ||
      e.key === "Home" ||
      e.key === "End" ||
      e.key === "Enter" ||
      e.key === "Escape" ||
      e.ctrlKey ||
      e.metaKey
    ) {
      return;
    }

    if (allowNegative === true && e.key === "-") {
      const target = e.target as HTMLInputElement;
      if (
        currentVal.includes("-") ||
        (target.selectionStart !== null && target.selectionStart > 0)
      ) {
        e.preventDefault();
      }
      return;
    }

    // Allow only one decimal point or comma if currency has scale > 0
    if (e.key === "." || e.key === ",") {
      if (scale <= 0 || currentVal.includes(".")) {
        e.preventDefault();
      }
      return;
    }

    // Reject all non-digits
    if (!/^[0-9]$/.test(e.key)) {
      e.preventDefault();
    }
  }

  function handleAmountInput(e: Event, setVal: (v: string) => void, allowNegative = false) {
    const target = e.target as HTMLInputElement;
    let val = target.value.replace(/,/g, ".");
    const isNeg = allowNegative === true && val.startsWith("-");
    val = val.replace(allowNegative === true ? /[^0-9.-]/g : /[^0-9.]/g, "");
    if (allowNegative === true) {
      val = (isNeg ? "-" : "") + val.replace(/-/g, "");
    }
    const hasMinus = val.startsWith("-");
    const rawDigits = hasMinus ? val.slice(1) : val;
    const parts = rawDigits.split(".");
    if (parts.length > 2) {
      val = (hasMinus ? "-" : "") + parts[0] + "." + parts.slice(1).join("");
    }
    const maxFrac = scale > 0 ? scale : 0;
    if (scale <= 0) {
      val = (hasMinus ? "-" : "") + (parts[0] ?? "");
    } else if (parts.length === 2 && parts[1] !== undefined && parts[1].length > maxFrac) {
      val = (hasMinus ? "-" : "") + parts[0] + "." + parts[1].slice(0, maxFrac);
    }
    setVal(val);
    target.value = val;
  }

  async function handleSave() {
    errorMessage = null;

    const idNum = Number(selectedOwnerId);
    if (selectedOwnerId === "" || Number.isInteger(idNum) === false || idNum <= 0) {
      errorMessage = "Please select a member.";
      return;
    }
    if (name.trim().length === 0) {
      errorMessage = "Account name is required.";
      return;
    }

    const ownerId = MemberId(idNum);
    const balanceUnits = parseMoneyInput(availableBalance, scale);
    const limitUnits = parseMoneyInput(creditLimit, scale);
    const availUnits = parseMoneyInput(availableCredit, scale);

    try {
      if (accountType === "bank_account") {
        if (isEdit === true && account !== null && account !== undefined) {
          await familyStore.updateBankAccount(account.id, {
            name: name.trim(),
            availableBalance: balanceUnits,
          });
        } else {
          await familyStore.addBankAccount({
            ownerMemberId: ownerId,
            name: name.trim(),
            availableBalance: balanceUnits,
          });
        }
      } else {
        if (isEdit === true && account !== null && account !== undefined) {
          await familyStore.updateCreditCard(account.id, {
            name: name.trim(),
            creditLimit: limitUnits,
            availableCredit: availUnits,
          });
        } else {
          await familyStore.addCreditCard({
            ownerMemberId: ownerId,
            name: name.trim(),
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
                ? 'bg-foreground text-background shadow-xs'
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
                ? 'bg-foreground text-background shadow-xs'
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
            Account Owner <span class="text-destructive">*</span>
          </label>
          <Select.Root bind:value={selectedOwnerId} type="single">
            <Select.Trigger
              id="owner-select"
              class="border-border/40 bg-background hover:bg-muted/30 focus-visible:border-foreground/30 focus-visible:ring-foreground/20 w-full focus-visible:ring-1"
            >
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

      <!-- Account Name -->
      <InputField
        id="account-name-input"
        label="Account Name"
        required
        bind:value={name}
        placeholder={accountType === "bank_account"
          ? "e.g. Chase Checking, Emergency Fund"
          : "e.g. Amex Gold, Sapphire Preferred"}
      />

      <!-- Available Balance (Bank Account) -->
      {#if accountType === "bank_account"}
        <InputField
          id="balance-input"
          label={`Available (${currencySymbol})`}
          required
          inputmode="decimal"
          placeholder={zeroPlaceholder}
          bind:value={availableBalance}
          inputClass="font-mono"
          onkeydown={(e) => handleAmountKeyDown(e, availableBalance, true)}
          oninput={(e) => handleAmountInput(e, (v) => (availableBalance = v), true)}
        />
      {/if}

      <!-- Credit Card Fields -->
      {#if accountType === "credit_card"}
        <div class="grid grid-cols-2 gap-3">
          <InputField
            id="credit-limit-input"
            label={`Credit Limit (${currencySymbol})`}
            required
            inputmode="decimal"
            placeholder={zeroPlaceholder}
            bind:value={creditLimit}
            inputClass="font-mono"
            onkeydown={(e) => handleAmountKeyDown(e, creditLimit, false)}
            oninput={(e) => handleAmountInput(e, (v) => (creditLimit = v), false)}
          />

          <InputField
            id="available-credit-input"
            label={`Available Credit (${currencySymbol})`}
            required
            inputmode="decimal"
            placeholder={zeroPlaceholder}
            bind:value={availableCredit}
            inputClass="font-mono"
            onkeydown={(e) => handleAmountKeyDown(e, availableCredit, false)}
            oninput={(e) => handleAmountInput(e, (v) => (availableCredit = v), false)}
          />
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
