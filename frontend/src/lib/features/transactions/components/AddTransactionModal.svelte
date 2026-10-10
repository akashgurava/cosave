<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import InputField from "$lib/components/InputField.svelte";
  import SelectField from "$lib/components/SelectField.svelte";
  import DateField from "$lib/components/DateField.svelte";
  import { AddTransactionForm } from "./addTransactionForm.svelte";
  import type { TransactionTypeItem } from "$lib/features/categories/types";
  import type { Member, Account, CurrencyOption } from "$lib/features/family/types";
  import type { Transaction } from "../types";

  interface Props {
    open: boolean;
    types: readonly TransactionTypeItem[];
    members: readonly Member[];
    accounts: readonly Account[];
    currency?: CurrencyOption;
    onAddTransaction: (newTx: Omit<Transaction, "id">) => void;
  }

  let {
    open = $bindable(false),
    types,
    members,
    accounts,
    currency,
    onAddTransaction,
  }: Props = $props();

  const form = new AddTransactionForm({
    getTypes: () => types,
    getMembers: () => members,
    getAccounts: () => accounts,
    onAddTransaction: (payload) => onAddTransaction(payload),
    close: () => {
      open = false;
    },
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="bg-card border-border/40 space-y-4 p-5 sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="text-foreground text-base font-semibold">Add Transaction</Dialog.Title>
      <Dialog.Description class="sr-only">Add transaction</Dialog.Description>
    </Dialog.Header>

    {#if form.error}
      <div
        class="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border p-2 text-xs"
      >
        {form.error}
      </div>
    {/if}

    <form
      novalidate
      onsubmit={(e) => {
        e.preventDefault();
        form.submit();
      }}
      class="space-y-3"
    >
      <!-- Row 1: Member & Account -->
      <div class="grid grid-cols-2 gap-3">
        <SelectField
          id="modal-new-tx-member"
          label="Member"
          required
          items={form.memberOptions}
          bind:value={form.memberId}
        />

        <SelectField
          id="modal-new-tx-account"
          label="Account"
          required
          disabled={form.accountOptions.length === 0}
          placeholder={form.accountOptions.length === 0 ? "No accounts found" : "Select account"}
          items={form.accountOptions}
          bind:value={form.accountId}
          contentClass="w-64"
        />
      </div>

      <!-- Row 2: Date & Transaction Type -->
      <div class="grid grid-cols-2 gap-3">
        <DateField
          id="modal-new-tx-date"
          label="Date"
          required
          bind:value={form.date}
        />

        <SelectField
          id="modal-new-tx-type"
          label="Transaction Type"
          required
          items={form.typeOptions}
          bind:value={form.typeId}
        />
      </div>

      <!-- Row 3: Category & Subcategory -->
      <div class="grid grid-cols-2 gap-3">
        <SelectField
          id="modal-new-tx-category"
          label="Category"
          required
          items={form.categoryOptions}
          bind:value={form.categoryId}
          onchange={form.handleCategoryChange}
        />

        <SelectField
          id="modal-new-tx-subcategory"
          label="Subcategory"
          allowEmpty
          emptyLabel="(None)"
          items={form.subcategoryOptions}
          bind:value={form.subcategoryId}
        />
      </div>

      <!-- Row 4: Amount & Description -->
      <div class="grid grid-cols-2 gap-3">
        <InputField
          bind:this={form.amountField}
          id="modal-new-tx-amount"
          label={`Amount (${currency?.symbol ?? "$"})`}
          required
          isAmount
          bind:value={form.amountStr}
          inputClass="h-8 text-xs"
        />

        <InputField
          id="modal-new-tx-description"
          label="Description"
          placeholder="e.g. WHOLEFDS SOMA #10294"
          bind:value={form.description}
          inputClass="h-8 text-xs"
        />
      </div>

      <!-- Row 5: Payee -->
      <InputField
        id="modal-new-tx-payee"
        label="Payee"
        placeholder="e.g. Whole Foods Market"
        bind:value={form.payee}
        inputClass="h-8 text-xs"
      />

      <Dialog.Footer class="flex items-center justify-end gap-2 pt-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          class="h-8 text-xs"
          onclick={() => (open = false)}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          size="sm"
          class="bg-foreground text-background hover:bg-foreground/90 h-8 text-xs font-medium"
        >
          Add Transaction
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
