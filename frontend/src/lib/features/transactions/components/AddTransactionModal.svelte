<script lang="ts">
  import { SvelteMap } from "svelte/reactivity";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import type { Transaction } from "../types";
  import { parseCurrencyInput } from "../mock";
  import type { CategoryItem, TransactionTypeItem } from "$lib/features/categories/types";
  import type { Member, Account, CurrencyOption } from "$lib/features/family/types";
  import {
    expectPresent,
    type MemberId,
    type AccountId,
    type CategoryId,
    type SubcategoryId,
    type MinorUnits,
    type TypeId,
  } from "$lib/types/core";

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

  let newDate = $state("2026-10-05");
  let newDescription = $state("");
  let newPayee = $state("");
  let newAmountStr = $state("");
  let newTypeId = $state<TypeId | number>(2 as TypeId);
  let newMemberId = $state<MemberId | number>(1 as MemberId);
  let newAccountId = $state<AccountId | number>(1 as AccountId);
  let newCategoryId = $state<CategoryId | number>(1 as CategoryId);
  let newSubcategoryId = $state<SubcategoryId | number | undefined>(undefined);
  let addModalError = $state<string | null>(null);

  // Initialize or align default selections when live metadata is loaded
  $effect(() => {
    if (types.length > 0 && !types.some((t) => t.id === newTypeId)) {
      const defaultType = types.find((t) => t.name.toLowerCase() === "expense") ?? types[0];
      if (defaultType) {
        newTypeId = defaultType.id;
      }
    }
  });

  $effect(() => {
    if (members.length > 0 && !members.some((m) => m.id === newMemberId)) {
      const firstMember = members[0];
      if (firstMember) {
        newMemberId = firstMember.id;
      }
    }
  });

  // Dynamic accounts available for selected member
  const modalAvailableAccounts = $derived.by(() => {
    return accounts.filter((a) => a.ownerMemberId === newMemberId);
  });

  const typeMap = $derived(new SvelteMap(types.map((t) => [t.id, t])));
  const categoryMap = $derived.by(() => {
    const map = new SvelteMap<CategoryId, CategoryItem>();
    for (const t of types) {
      for (const c of t.categories) {
        map.set(c.id as CategoryId, c);
      }
    }
    return map;
  });

  // Dynamic categories available for selected type
  const modalAvailableCategories = $derived.by(() => {
    const selectedType = typeMap.get(newTypeId as TypeId);
    return selectedType?.categories ?? [];
  });

  // Dynamic subcategories available for selected category
  const modalAvailableSubcategories = $derived.by(() => {
    const selectedCat = categoryMap.get(newCategoryId as CategoryId);
    return selectedCat?.subcategories ?? [];
  });

  // Update account when member changes if account doesn't belong to member
  $effect(() => {
    const acc = modalAvailableAccounts[0];
    if (acc && !modalAvailableAccounts.some((a) => a.id === newAccountId)) {
      newAccountId = acc.id;
    }
  });

  // Update category when type changes if category doesn't belong to type
  $effect(() => {
    const cat = modalAvailableCategories[0];
    if (cat && !modalAvailableCategories.some((c) => c.id === newCategoryId)) {
      newCategoryId = cat.id;
      const sub = cat.subcategories[0];
      newSubcategoryId = sub ? sub.id : undefined;
    }
  });

  function handleSubmit() {
    addModalError = null;

    if (!newDescription.trim() && !newPayee.trim()) {
      addModalError = "Statement description or payee is required.";
      return;
    }

    const parsedAmount = parseCurrencyInput(newAmountStr);
    if (parsedAmount === null || parsedAmount <= 0) {
      addModalError = "Please enter a valid amount greater than 0.00.";
      return;
    }

    if (!newDate) {
      addModalError = "Transaction date is required.";
      return;
    }

    const selectedType = expectPresent(
      typeMap.get(newTypeId as TypeId),
      "VIEW.ADD_TRANSACTION_MODAL.RESOLVE_TYPE",
      `Transaction type ${newTypeId} not found in available types`,
    );

    onAddTransaction({
      date: newDate,
      description: newDescription.trim() || newPayee.trim(),
      payee: newPayee.trim(),
      amount: parsedAmount as MinorUnits,
      typeId: selectedType.id as TypeId,
      type: selectedType.name,
      typeColor: selectedType.color,
      memberId: newMemberId as MemberId,
      accountId: newAccountId as AccountId,
      categoryId: newCategoryId as CategoryId,
      subcategoryId:
        newSubcategoryId !== undefined && Number(newSubcategoryId) > 0
          ? (newSubcategoryId as SubcategoryId)
          : undefined,
      status: "cleared",
    });

    // Reset and close
    newDescription = "";
    newPayee = "";
    newAmountStr = "";
    open = false;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="bg-card border-border/40 space-y-4 p-5 sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="text-foreground text-base font-semibold">Add Transaction</Dialog.Title>
      <Dialog.Description class="sr-only">Add transaction</Dialog.Description>
    </Dialog.Header>

    {#if addModalError}
      <div
        class="border-destructive/30 bg-destructive/10 text-destructive rounded-lg border p-2 text-xs"
      >
        {addModalError}
      </div>
    {/if}

    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
      class="space-y-3"
    >
      <!-- Row 1: Flow Type & Date -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label
            for="modal-new-tx-type"
            class="text-muted-foreground font-mono text-[10px] uppercase">Flow Type *</label
          >
          <select
            id="modal-new-tx-type"
            bind:value={newTypeId}
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 text-xs"
          >
            {#each types as t (t.id)}
              <option value={t.id}>{t.name}</option>
            {/each}
          </select>
        </div>

        <div class="space-y-1">
          <label
            for="modal-new-tx-date"
            class="text-muted-foreground font-mono text-[10px] uppercase">Date *</label
          >
          <input
            id="modal-new-tx-date"
            type="date"
            bind:value={newDate}
            required
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 font-mono text-xs"
          />
        </div>
      </div>

      <!-- Row 2: Member & Member-Scoped Account -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label
            for="modal-new-tx-member"
            class="text-muted-foreground font-mono text-[10px] uppercase">Member *</label
          >
          <select
            id="modal-new-tx-member"
            bind:value={newMemberId}
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 text-xs"
          >
            {#each members as m (m.id)}
              <option value={m.id}>{m.memberName}</option>
            {/each}
          </select>
        </div>

        <div class="space-y-1">
          <label
            for="modal-new-tx-account"
            class="text-muted-foreground font-mono text-[10px] uppercase">Account *</label
          >
          <select
            id="modal-new-tx-account"
            bind:value={newAccountId}
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 text-xs"
          >
            {#each modalAvailableAccounts as acc (acc.id)}
              <option value={acc.id}>
                {acc.type === "bank_account" ? acc.accountName : acc.cardName} ({acc.bankName} ····{acc.last4})
              </option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Row 3: Category & Optional Subcategory -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label
            for="modal-new-tx-category"
            class="text-muted-foreground font-mono text-[10px] uppercase">Category *</label
          >
          <select
            id="modal-new-tx-category"
            bind:value={newCategoryId}
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 text-xs"
          >
            {#each modalAvailableCategories as cat (cat.id)}
              <option value={cat.id}>{cat.name}</option>
            {/each}
          </select>
        </div>

        <div class="space-y-1">
          <label
            for="modal-new-tx-subcategory"
            class="text-muted-foreground font-mono text-[10px] uppercase"
            >Subcategory (Optional)</label
          >
          <select
            id="modal-new-tx-subcategory"
            bind:value={newSubcategoryId}
            class="border-border/40 bg-background w-full rounded-md border px-2.5 py-1.5 text-xs"
          >
            <option value={undefined}>(None)</option>
            {#each modalAvailableSubcategories as sub (sub.id)}
              <option value={sub.id}>{sub.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Row 4: Description & Amount -->
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1">
          <label
            for="modal-new-tx-description"
            class="text-muted-foreground font-mono text-[10px] uppercase">Description *</label
          >
          <Input
            id="modal-new-tx-description"
            type="text"
            placeholder="e.g. WHOLEFDS SOMA #10294"
            bind:value={newDescription}
            required
            class="border-border/40 bg-background h-8 font-mono text-xs"
          />
        </div>

        <div class="space-y-1">
          <label
            for="modal-new-tx-amount"
            class="text-muted-foreground font-mono text-[10px] uppercase"
            >Amount ({currency?.symbol ?? "$"}) *</label
          >
          <Input
            id="modal-new-tx-amount"
            type="text"
            placeholder="0.00"
            bind:value={newAmountStr}
            required
            class="border-border/40 bg-background h-8 font-mono text-xs"
          />
        </div>
      </div>

      <!-- Row 5: Payee (Optional counterparty) -->
      <div class="space-y-1">
        <label
          for="modal-new-tx-payee"
          class="text-muted-foreground font-mono text-[10px] uppercase"
          >Payee (Optional Counterparty)</label
        >
        <Input
          id="modal-new-tx-payee"
          type="text"
          placeholder="e.g. Whole Foods Market"
          bind:value={newPayee}
          class="border-border/40 bg-background h-8 text-xs"
        />
      </div>

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
