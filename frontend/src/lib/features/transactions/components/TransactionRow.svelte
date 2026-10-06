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
  import * as Table from "$lib/components/ui/table";
  import * as Popover from "$lib/components/ui/popover";
  import { AmountDisplay } from "$lib/components";
  import type { Transaction } from "../types";
  import { parseCurrencyInput } from "../mock";
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
    type CategoryId,
    type SubcategoryId,
    type TypeId,
  } from "$lib/types/core";

  interface Props {
    tx: Transaction;
    draft?: Partial<Transaction>;
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

  const effectiveTx = $derived<Transaction>(
    draft && Object.keys(draft).length > 0 ? { ...tx, ...draft } : tx,
  );

  const memberMap = $derived(new Map(members.map((m) => [m.id, m])));
  const accountMap = $derived(new Map(accounts.map((a) => [a.id, a])));
  const currencyMap = $derived(new Map(currencies.map((c) => [c.id, c])));
  const typeMap = $derived(new Map(types.map((t) => [t.id, t])));

  const member = $derived<Member>(
    expectPresent(
      memberMap.get(effectiveTx.memberId),
      "VIEW.TRANSACTION_ROW.RESOLVE_MEMBER",
      `Member ${effectiveTx.memberId} not found for transaction ${tx.id}`,
    ),
  );

  const account = $derived<Account>(
    expectPresent(
      accountMap.get(effectiveTx.accountId),
      "VIEW.TRANSACTION_ROW.RESOLVE_ACCOUNT",
      `Account ${effectiveTx.accountId} not found for transaction ${tx.id}`,
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

  // Cell popover states
  let activePopover = $state<"date" | "account" | "category" | null>(null);

  // In-place text editing state
  let editingCell = $state<"payee" | "amount" | null>(null);
  let editStringValue = $state("");

  function autoFocus(node: HTMLElement) {
    node.focus();
  }

  function startEditing(field: "payee" | "amount", initialValue: string) {
    editingCell = field;
    editStringValue = initialValue;
  }

  function stageEditingPayee() {
    onDraftChange(tx.id, { payee: editStringValue.trim() });
    editingCell = null;
  }

  function stageEditingAmount() {
    const parsed = parseCurrencyInput(editStringValue);
    if (parsed !== null && parsed > 0) {
      onDraftChange(tx.id, { amount: parsed as MinorUnits });
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

  function handleSave() {
    if (editingCell === "payee") {
      onDraftChange(tx.id, { payee: editStringValue.trim() });
      editingCell = null;
    } else if (editingCell === "amount") {
      const parsed = parseCurrencyInput(editStringValue);
      if (parsed !== null && parsed > 0) {
        onDraftChange(tx.id, { amount: parsed as MinorUnits });
      }
      editingCell = null;
    }
    onSave(tx.id);
  }

  function handleDiscard() {
    editingCell = null;
    onDiscard(tx.id);
  }

  // Description view & copy state
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

  // Delete confirmation state (armed on first click, double-click deletes, outside click cancels)
  let confirmingDelete = $state(false);
  let deleteConfirmTimeoutId: ReturnType<typeof setTimeout> | null = null;

  function handleDeleteClick() {
    activeDescription = false;
    if (confirmingDelete) {
      if (deleteConfirmTimeoutId) clearTimeout(deleteConfirmTimeoutId);
      confirmingDelete = false;
      onDelete(tx.id);
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
        (el) => el instanceof HTMLElement && el.getAttribute("data-delete-btn") === String(tx.id),
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
      editingCell = null;
    }
  }}
/>

<Table.Row
  class="group hover:bg-muted/20 border-border/20 h-10 border-b transition-colors {isDirty
    ? 'bg-emerald-500/5'
    : ''}"
>
  <!-- Column 1: Date (In-Place Date Popover, auto-dismiss on change) -->
  <Table.Cell class="w-24 overflow-hidden py-1 pr-2 pl-3 whitespace-nowrap">
    <Popover.Root
      open={activePopover === "date"}
      onOpenChange={(open) => {
        activePopover = open ? "date" : null;
      }}
    >
      <Popover.Trigger
        class="text-muted-foreground hover:bg-muted/60 hover:text-foreground w-full rounded px-1 py-0.5 text-left font-mono text-[11px] transition-colors"
        title="Click to edit date"
      >
        {effectiveTx.date}
      </Popover.Trigger>
      <Popover.Content align="start" side="bottom" sideOffset={4} class="w-44 p-2">
        <input
          type="date"
          value={effectiveTx.date}
          onchange={(e) => {
            onDraftChange(tx.id, { date: e.currentTarget.value });
            activePopover = null;
          }}
          class="border-border/40 bg-background w-full rounded border px-2 py-1 font-mono text-xs"
        />
      </Popover.Content>
    </Popover.Root>
  </Table.Cell>

  <!-- Column 2: Account (Merged Member › Account Breadcrumb, anchored popover, auto-dismiss) -->
  <Table.Cell class="w-44 overflow-hidden px-2 py-1 whitespace-nowrap">
    <Popover.Root
      open={activePopover === "account"}
      onOpenChange={(open) => {
        activePopover = open ? "account" : null;
      }}
    >
      <Popover.Trigger
        class="text-foreground hover:bg-muted/60 flex w-full max-w-full items-center gap-1.5 truncate rounded px-1.5 py-0.5 text-left text-[11px] transition-colors"
        title="{member.memberName}: {account.type === 'bank_account'
          ? account.accountName
          : account.cardName}"
      >
        <span class="text-foreground shrink-0 font-medium">{member.memberName}</span>
        <span class="text-muted-foreground/40 font-mono">›</span>
        <span class="text-foreground truncate font-medium">
          {account.type === "bank_account" ? account.accountName : account.cardName}
        </span>
      </Popover.Trigger>
      <Popover.Content align="start" side="bottom" sideOffset={4} class="w-68 space-y-1 p-1.5">
        <div class="max-h-60 space-y-2 overflow-y-auto">
          {#each members as m (m.id)}
            <div>
              <div
                class="text-muted-foreground/70 px-1 py-0.5 font-mono text-[9px] font-semibold uppercase"
              >
                {m.memberName}
              </div>
              <div class="space-y-0.5">
                {#each accounts.filter((a) => a.ownerMemberId === m.id) as acc (acc.id)}
                  <button
                    type="button"
                    onclick={() => {
                      onDraftChange(tx.id, {
                        memberId: m.id,
                        accountId: acc.id,
                      });
                      activePopover = null;
                    }}
                    class="hover:bg-muted/50 flex w-full items-center justify-between rounded p-1.5 text-left text-xs transition-colors"
                  >
                    <div class="mr-2 truncate">
                      <div class="truncate font-medium">
                        {acc.type === "bank_account" ? acc.accountName : acc.cardName}
                      </div>
                      <div class="text-muted-foreground font-mono text-[9px]">
                        {acc.bankName} ····{acc.last4}
                      </div>
                    </div>
                    {#if effectiveTx.memberId === m.id && effectiveTx.accountId === acc.id}
                      <CheckIcon class="size-3 shrink-0 text-emerald-500" />
                    {/if}
                  </button>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </Popover.Content>
    </Popover.Root>
  </Table.Cell>

  <!-- Column 3: Category (Merged Type › Category › Subcategory Breadcrumb, anchored popover, auto-dismiss) -->
  <Table.Cell class="min-w-48 overflow-hidden px-2 py-1 whitespace-nowrap">
    <Popover.Root
      open={activePopover === "category"}
      onOpenChange={(open) => {
        activePopover = open ? "category" : null;
      }}
    >
      <Popover.Trigger
        class="text-foreground hover:bg-muted/60 flex w-full max-w-full items-center gap-1.5 truncate rounded px-1.5 py-0.5 text-left text-[11px] transition-colors"
        title="{effectiveTx.type} › {category.name}{subcategory ? ` › ${subcategory.name}` : ''}"
      >
        <span class="text-foreground shrink-0 font-medium capitalize">{effectiveTx.type}</span>
        <span class="text-muted-foreground/40 font-mono">›</span>
        <span class="text-foreground shrink-0 font-medium">{category.name}</span>
        {#if subcategory}
          <span class="text-muted-foreground/40 font-mono">›</span>
          <span class="text-muted-foreground shrink-0">{subcategory.name}</span>
        {/if}
      </Popover.Trigger>
      <Popover.Content align="start" side="bottom" sideOffset={4} class="w-72 space-y-1.5 p-1.5">
        <!-- Dynamic flow type selector buttons -->
        <div class="border-border/40 flex flex-wrap items-center gap-1 border-b pb-1.5">
          {#each types as t (t.id)}
            <button
              type="button"
              onclick={() => {
                const defaultCat = t.categories[0];
                onDraftChange(tx.id, {
                  typeId: t.id as TypeId,
                  type: t.name,
                  typeColor: t.color,
                  categoryId: (defaultCat?.id ?? tx.categoryId) as CategoryId,
                  subcategoryId: undefined,
                });
              }}
              class="flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors {effectiveTx.typeId ===
                t.id || effectiveTx.type.toLowerCase() === t.name.toLowerCase()
                ? 'bg-muted text-foreground font-semibold'
                : 'text-muted-foreground hover:bg-muted/50'}"
            >
              <span class="size-1.5 rounded-full" style="background-color: {t.color};"></span>
              <span>{t.name}</span>
            </button>
          {/each}
        </div>

        <!-- Categories for selected type -->
        <div class="max-h-60 space-y-1 overflow-y-auto">
          {#each txType.categories as cat (cat.id)}
            <div class="space-y-0.5">
              <button
                type="button"
                onclick={() => {
                  onDraftChange(tx.id, {
                    categoryId: cat.id as CategoryId,
                    typeId: txType.id as TypeId,
                    type: txType.name,
                    typeColor: txType.color,
                    subcategoryId: undefined,
                  });
                  activePopover = null;
                }}
                class="hover:bg-muted/50 flex w-full items-center justify-between rounded p-1.5 text-xs transition-colors {effectiveTx.categoryId ===
                  cat.id && !effectiveTx.subcategoryId
                  ? 'bg-muted/40 font-medium'
                  : ''}"
              >
                <span class="flex items-center gap-1.5 truncate">
                  <span
                    class="size-2 shrink-0 rounded-full"
                    style="background-color: {effectiveTypeColor};"
                  ></span>
                  <span class="truncate">{cat.name}</span>
                </span>
                {#if effectiveTx.categoryId === cat.id && !effectiveTx.subcategoryId}
                  <CheckIcon class="size-3 shrink-0 text-emerald-500" />
                {/if}
              </button>
              {#if cat.subcategories.length > 0}
                <div class="space-y-0.5 pl-4">
                  {#each cat.subcategories as sub (sub.id)}
                    <button
                      type="button"
                      onclick={() => {
                        onDraftChange(tx.id, {
                          categoryId: cat.id as CategoryId,
                          typeId: txType.id as TypeId,
                          type: txType.name,
                          typeColor: txType.color,
                          subcategoryId: sub.id as SubcategoryId,
                        });
                        activePopover = null;
                      }}
                      class="hover:bg-muted/50 text-muted-foreground hover:text-foreground flex w-full items-center justify-between rounded px-2 py-1 text-[11px] transition-colors {effectiveTx.categoryId ===
                        cat.id && effectiveTx.subcategoryId === sub.id
                        ? 'text-foreground font-medium'
                        : ''}"
                    >
                      <span class="truncate">{sub.name}</span>
                      {#if effectiveTx.categoryId === cat.id && effectiveTx.subcategoryId === sub.id}
                        <CheckIcon class="size-3 shrink-0 text-emerald-500" />
                      {/if}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </Popover.Content>
    </Popover.Root>
  </Table.Cell>

  <!-- Column 4: Payee (Double-click/click-to-edit inline cell) -->
  <Table.Cell class="w-40 overflow-hidden px-2 py-1 whitespace-nowrap">
    {#if editingCell === "payee"}
      <input
        use:autoFocus
        type="text"
        bind:value={editStringValue}
        onblur={stageEditingPayee}
        onkeydown={(e) => handleKeyDown(e, "payee")}
        class="border-border/40 bg-background text-foreground h-6 w-full rounded border px-1.5 text-xs outline-none"
      />
    {:else}
      <button
        type="button"
        onclick={() => startEditing("payee", effectiveTx.payee)}
        class="hover:bg-muted/60 text-foreground block w-full truncate rounded px-1.5 py-0.5 text-left text-xs transition-colors"
        title="Click to edit payee (counterparty)"
      >
        {#if effectiveTx.payee.trim().length > 0}
          <span class="truncate font-normal">{effectiveTx.payee}</span>
        {:else}
          <span class="text-muted-foreground/40 italic">Add payee...</span>
        {/if}
      </button>
    {/if}
  </Table.Cell>

  <!-- Column 5: Amount (Inline Click-to-Edit Cell with AmountDisplay) -->
  <Table.Cell class="w-28 overflow-hidden px-2 py-1 text-right whitespace-nowrap">
    {#if editingCell === "amount"}
      <input
        use:autoFocus
        type="text"
        bind:value={editStringValue}
        onblur={stageEditingAmount}
        onkeydown={(e) => handleKeyDown(e, "amount")}
        class="border-border/40 bg-background text-foreground h-6 w-full rounded border px-1.5 text-right font-mono text-xs outline-none"
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
    <div class="flex items-center justify-end gap-0.5">
      <!-- 1. Status Icon -->
      <button
        onclick={() =>
          onDraftChange(tx.id, {
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
              if (activeDescription) {
                e.preventDefault();
                copyDescription(effectiveTx.description);
              }
            }}
            class="hover:bg-muted inline-flex size-6 items-center justify-center rounded transition-colors {activeDescription
              ? 'bg-muted/80 text-foreground'
              : 'text-muted-foreground/70 hover:text-foreground'}"
            title={effectiveTx.description}
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
          disabled
          class="text-muted-foreground/20 inline-flex size-6 cursor-not-allowed items-center justify-center rounded"
          title="No statement description"
        >
          <FileTextIcon class="size-3.5" />
        </button>
      {/if}

      <!-- 3. Save Floppy Icon -->
      <button
        disabled={!isDirty}
        onclick={handleSave}
        class="inline-flex size-6 items-center justify-center rounded transition-colors {isDirty
          ? 'cursor-pointer text-emerald-500 hover:bg-emerald-500/15'
          : 'text-muted-foreground/20 cursor-not-allowed'}"
        title={isDirty ? "Save changes" : "No unsaved changes"}
      >
        <SaveIcon class="size-3.5" />
      </button>

      <!-- 4. Discard X Icon -->
      <button
        disabled={!isDirty}
        onclick={handleDiscard}
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
          data-delete-btn={tx.id}
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
          data-delete-btn={tx.id}
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
  </Table.Cell>
</Table.Row>
