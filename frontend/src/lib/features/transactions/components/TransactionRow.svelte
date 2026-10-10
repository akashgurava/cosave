<script lang="ts">
  import * as Table from "$lib/components/ui/table";
  import { AmountDisplay, DateField, InputField } from "$lib/components";
  import type { Transaction } from "../types";
  import { parseCurrencyInput } from "../filters";
  import type {
    CategoryItem,
    SubcategoryItem,
    TransactionTypeItem,
  } from "$lib/features/categories/types";
  import type { Member, Account, CurrencyOption } from "$lib/features/family/types";
  import {
    expectPresent,
    type TransactionId,
    type MinorUnits,
  } from "$lib/types";
  import AccountCell from "./row/AccountCell.svelte";
  import CategoryCell from "./row/CategoryCell.svelte";
  import ActionCell from "./row/ActionCell.svelte";

  interface Props {
    tx: Transaction;
    draft?: Transaction;
    isDirty: boolean;
    types: readonly TransactionTypeItem[];
    members: readonly Member[];
    accounts: readonly Account[];
    currencies: readonly CurrencyOption[];
    onSave: (id: TransactionId) => void;
    onDiscard: (id: TransactionId) => void;
    onDelete: (id: TransactionId) => void;
    onDraftChange: (id: TransactionId, updates: Partial<Transaction>) => void;
  }

  let {
    tx,
    draft,
    isDirty,
    types,
    members,
    accounts,
    currencies,
    onSave,
    onDiscard,
    onDelete,
    onDraftChange,
  }: Props = $props();

  const effectiveTx = $derived<Transaction>(draft !== undefined ? draft : tx);

  const memberMap = $derived(new Map(members.map((m) => [m.id, m])));
  const accountMap = $derived(new Map(accounts.map((a) => [a.id, a])));
  const currencyMap = $derived(new Map(currencies.map((c) => [c.id, c])));
  const typeMap = $derived(new Map(types.map((t) => [t.id, t])));

  const account = $derived<Account>(
    expectPresent(
      accountMap.get(effectiveTx.accountId),
      "VIEW.TRANSACTION_ROW.RESOLVE_ACCOUNT",
      `Account ${effectiveTx.accountId} not found for transaction ${tx.id}`,
    ),
  );

  const member = $derived<Member>(
    expectPresent(
      memberMap.get(account.ownerMemberId),
      "VIEW.TRANSACTION_ROW.RESOLVE_MEMBER",
      `Member for account ${effectiveTx.accountId} not found for transaction ${tx.id}`,
    ),
  );

  const accountCurrency = $derived<CurrencyOption>(
    expectPresent(
      currencyMap.get(account.currencyId),
      "VIEW.TRANSACTION_ROW.RESOLVE_CURRENCY",
      `Currency ${account.currencyId} not found for account ${account.id}`,
    ),
  );

  const txType = $derived<TransactionTypeItem>(
    expectPresent(
      typeMap.get(effectiveTx.typeId),
      "VIEW.TRANSACTION_ROW.RESOLVE_TYPE",
      `Type ${effectiveTx.typeId} not found for transaction ${tx.id}`,
    ),
  );

  const effectiveTypeColor = $derived<string>(txType.color);

  const category = $derived.by<CategoryItem>(() => {
    const found = txType.categories.find((c) => c.id === effectiveTx.categoryId);
    return expectPresent(
      found,
      "VIEW.TRANSACTION_ROW.RESOLVE_CATEGORY",
      `Category ${effectiveTx.categoryId} not found under type ${txType.name} for transaction ${tx.id}`,
    );
  });

  const subcategory = $derived.by<SubcategoryItem | undefined>(() => {
    if (!effectiveTx.subcategoryId || Number(effectiveTx.subcategoryId) <= 0) return undefined;
    const found = category.subcategories.find((s) => s.id === effectiveTx.subcategoryId);
    return expectPresent(
      found,
      "VIEW.TRANSACTION_ROW.RESOLVE_SUBCATEGORY",
      `Subcategory ${effectiveTx.subcategoryId} not found under category ${category.name} for transaction ${tx.id}`,
    );
  });

  // In-place text editing state
  let editingCell = $state<"payee" | "amount" | null>(null);
  let editStringValue = $state("");

  function startEditing(field: "payee" | "amount", initialValue: string) {
    editingCell = field;
    editStringValue = initialValue;
  }

  function stageEditingPayee() {
    const trimmed = editStringValue.trim();
    const currentPayee = effectiveTx.payee !== null ? effectiveTx.payee : "";
    if (trimmed !== currentPayee) {
      onDraftChange(tx.id, { payee: trimmed.length > 0 ? trimmed : null });
    }
    editingCell = null;
  }

  function stageEditingAmount() {
    const parsed = parseCurrencyInput(editStringValue);
    if (parsed !== null && parsed > 0) {
      if (parsed !== effectiveTx.amount) {
        onDraftChange(tx.id, { amount: parsed as MinorUnits });
      }
    }
    editingCell = null;
  }

  function handleKeyDown(e: KeyboardEvent, field: "payee" | "amount") {
    if (e.key === "Enter") {
      if (field === "payee") stageEditingPayee();
      if (field === "amount") stageEditingAmount();
    } else if (e.key === "Escape") {
      editingCell = null;
    }
  }

  function handleSave(id: TransactionId) {
    if (editingCell === "payee") {
      stageEditingPayee();
    } else if (editingCell === "amount") {
      stageEditingAmount();
    }
    onSave(id);
  }

  function handleDiscard(id: TransactionId) {
    editingCell = null;
    onDiscard(id);
  }
</script>

<Table.Row
  class="group hover:bg-muted/20 border-border/20 h-10 border-b transition-colors {isDirty
    ? 'bg-emerald-500/5'
    : ''}"
>
  <!-- Column 1: Date (In-Place Date Popover, auto-dismiss on change) -->
  <Table.Cell class="w-24 overflow-hidden py-1 pr-2 pl-3 whitespace-nowrap">
    <DateField
      value={effectiveTx.date}
      showIcon={false}
      onchange={(val) => onDraftChange(tx.id, { date: val })}
    />
  </Table.Cell>

  <!-- Column 2: Account (Merged Member › Account Breadcrumb, anchored popover, auto-dismiss) -->
  <Table.Cell class="w-44 overflow-hidden px-2 py-1 whitespace-nowrap">
    <AccountCell
      txId={tx.id}
      {effectiveTx}
      {member}
      {account}
      {members}
      {accounts}
      {onDraftChange}
    />
  </Table.Cell>

  <!-- Column 3: Category (Merged Type › Category › Subcategory Breadcrumb, anchored popover, auto-dismiss) -->
  <Table.Cell class="min-w-48 overflow-hidden px-2 py-1 whitespace-nowrap">
    <CategoryCell
      txId={tx.id}
      {effectiveTx}
      {txType}
      {effectiveTypeColor}
      {category}
      {subcategory}
      {types}
      {onDraftChange}
    />
  </Table.Cell>

  <!-- Column 4: Payee (Double-click/click-to-edit inline cell) -->
  <Table.Cell class="w-40 overflow-hidden px-2 py-1 whitespace-nowrap">
    {#if editingCell === "payee"}
      <InputField
        bind:value={editStringValue}
        onblur={stageEditingPayee}
        onkeydown={(e) => handleKeyDown(e, "payee")}
        autofocus
        inputClass="h-6 w-full rounded px-1.5 text-xs outline-none"
      />
    {:else}
      <button
        type="button"
        onclick={() => startEditing("payee", effectiveTx.payee !== null ? effectiveTx.payee : "")}
        class="hover:bg-muted/60 text-foreground block w-full truncate rounded px-1.5 py-0.5 text-left text-xs transition-colors"
        title="Click to edit payee (counterparty)"
      >
        {#if effectiveTx.payee !== null && effectiveTx.payee.trim().length > 0}
          <span class="truncate font-normal">{effectiveTx.payee}</span>
        {:else}
          <span class="text-muted-foreground/40 italic">Add payee...</span>
        {/if}
      </button>
    {/if}
  </Table.Cell>

  <!-- Column 5: Amount (Inline Click-to-Edit Cell with AmountDisplay, strictly numbers only) -->
  <Table.Cell class="w-28 overflow-hidden px-2 py-1 text-right whitespace-nowrap">
    {#if editingCell === "amount"}
      <InputField
        isAmount
        bind:value={editStringValue}
        onblur={stageEditingAmount}
        onkeydown={(e) => handleKeyDown(e, "amount")}
        autofocus
        inputClass="h-6 w-full rounded px-1.5 text-right font-mono text-xs outline-none"
      />
    {:else}
      <button
        type="button"
        onclick={() => startEditing("amount", (Math.abs(effectiveTx.amount) / 100).toFixed(2))}
        class="hover:bg-muted/60 block w-full rounded px-1 py-0.5 text-right text-xs transition-colors"
        title="Click to edit amount"
      >
        <AmountDisplay
          amount={effectiveTx.amount}
          currency={accountCurrency}
          color={effectiveTypeColor}
          class="font-medium"
        />
      </button>
    {/if}
  </Table.Cell>

  <!-- Column 6: Action & Status Icons (Status, Description, Save Floppy, Discard X, Delete) -->
  <Table.Cell class="w-40 overflow-visible px-2 py-1 text-right whitespace-nowrap">
    <ActionCell
      txId={tx.id}
      {effectiveTx}
      {isDirty}
      {onDraftChange}
      onSave={handleSave}
      onDiscard={handleDiscard}
      {onDelete}
    />
  </Table.Cell>
</Table.Row>
