<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import {
    ArrowUpIcon,
    ArrowDownIcon,
    ArrowUpDownIcon,
    ChevronDownIcon,
    ChevronRightIcon,
  } from "@lucide/svelte";
  import * as Table from "$lib/components/ui/table";
  import { AmountDisplay } from "$lib/components";
  import type { Transaction, SortField, SortDirection } from "../types";
  import type { TimelineGroup } from "../store.svelte";
  import type { TransactionTypeItem } from "$lib/features/categories/types";
  import type { Member, Account, CurrencyOption } from "$lib/features/family/types";
  import type { TransactionId } from "$lib/types";
  import TransactionRow from "./TransactionRow.svelte";

  interface Props {
    timelineGroups: readonly TimelineGroup[];
    sortField: SortField;
    sortDirection: SortDirection;
    types: readonly TransactionTypeItem[];
    members: readonly Member[];
    accounts: readonly Account[];
    currencies: readonly CurrencyOption[];
    baseCurrency: CurrencyOption;
    onSort: (field: SortField) => void;
    rowDrafts: Record<string, Partial<Transaction>>;
    hasRowDraft: (id: TransactionId) => boolean;
    onSaveRowDraft: (id: TransactionId) => void;
    onDiscardRowDraft: (id: TransactionId) => void;
    onDeleteTransaction: (id: TransactionId) => void;
    onDraftChange: (id: TransactionId, updates: Partial<Transaction>) => void;
  }

  let {
    timelineGroups,
    sortField,
    sortDirection,
    types,
    members,
    accounts,
    currencies,
    baseCurrency,
    onSort,
    rowDrafts,
    hasRowDraft,
    onSaveRowDraft,
    onDiscardRowDraft,
    onDeleteTransaction,
    onDraftChange,
  }: Props = $props();

  let collapsedGroupKeys = new SvelteSet<string>();

  function toggleGroupCollapse(key: string) {
    if (collapsedGroupKeys.has(key)) {
      collapsedGroupKeys.delete(key);
    } else {
      collapsedGroupKeys.add(key);
    }
  }
</script>

<div class="border-border/40 bg-card overflow-hidden rounded-xl border shadow-xs">
  <div class="overflow-x-auto">
    <Table.Root class="w-full min-w-215 table-fixed text-xs">
      <Table.Header class="bg-muted/40 border-border/40 border-b select-none">
        <Table.Row class="hover:bg-muted/40 h-9 font-mono">
          <!-- 1. Date -->
          <Table.Head class="w-24 cursor-pointer pr-2 pl-3" onclick={() => onSort("date")}>
            <div
              class="text-muted-foreground hover:text-foreground flex items-center gap-1 text-[10px] uppercase"
            >
              Date
              {#if sortField === "date"}
                {#if sortDirection === "asc"}<ArrowUpIcon class="size-3" />{:else}<ArrowDownIcon
                    class="size-3"
                  />{/if}
              {:else}
                <ArrowUpDownIcon class="size-3 opacity-40" />
              {/if}
            </div>
          </Table.Head>

          <!-- 2. Account -->
          <Table.Head class="w-44 px-2">
            <div class="text-muted-foreground text-[10px] uppercase">Account</div>
          </Table.Head>

          <!-- 3. Category (Biggest column, flexible with min-w) -->
          <Table.Head class="min-w-48 px-2">
            <div class="text-muted-foreground text-[10px] uppercase">Category</div>
          </Table.Head>

          <!-- 4. Payee -->
          <Table.Head class="w-40 cursor-pointer px-2" onclick={() => onSort("payee")}>
            <div
              class="text-muted-foreground hover:text-foreground flex items-center gap-1 text-[10px] uppercase"
            >
              Payee
              {#if sortField === "payee"}
                {#if sortDirection === "asc"}<ArrowUpIcon class="size-3" />{:else}<ArrowDownIcon
                    class="size-3"
                  />{/if}
              {:else}
                <ArrowUpDownIcon class="size-3 opacity-40" />
              {/if}
            </div>
          </Table.Head>

          <!-- 5. Amount -->
          <Table.Head
            class="w-28 cursor-pointer px-2 py-1 text-right"
            onclick={() => onSort("amount")}
          >
            <div
              class="text-muted-foreground hover:text-foreground flex items-center justify-end gap-1 text-[10px] uppercase"
            >
              Amount
              {#if sortField === "amount"}
                {#if sortDirection === "asc"}<ArrowUpIcon class="size-3" />{:else}<ArrowDownIcon
                    class="size-3"
                  />{/if}
              {:else}
                <ArrowUpDownIcon class="size-3 opacity-40" />
              {/if}
            </div>
          </Table.Head>

          <!-- 6. Actions (Status, Description, Save Floppy, Discard X, Delete) -->
          <Table.Head class="w-40 px-2 py-1 text-right"></Table.Head>
        </Table.Row>
      </Table.Header>

      <Table.Body>
        {#if timelineGroups.length === 0}
          <Table.Row>
            <Table.Cell colspan={6} class="text-muted-foreground py-12 text-center">
              No transactions match the selected filters.
            </Table.Cell>
          </Table.Row>
        {:else}
          {#each timelineGroups as group (group.dateKey)}
            <!-- Sticky Collapsible Timeline Section Header Row -->
            <Table.Row
              class="bg-muted/30 border-border/40 hover:bg-muted/50 cursor-pointer border-y transition-colors select-none"
              onclick={() => toggleGroupCollapse(group.dateKey)}
            >
              <Table.Cell colspan={6} class="px-3 py-1.5">
                <div class="flex items-center justify-between text-xs">
                  <div class="flex items-center gap-2 font-mono">
                    {#if collapsedGroupKeys.has(group.dateKey)}
                      <ChevronRightIcon
                        class="text-muted-foreground size-3.5 transition-transform"
                      />
                    {:else}
                      <ChevronDownIcon
                        class="text-muted-foreground size-3.5 transition-transform"
                      />
                    {/if}
                    <span class="size-1.5 rounded-full bg-emerald-500"></span>
                    <span class="text-foreground font-semibold">{group.title}</span>
                    <span
                      class="bg-muted text-muted-foreground rounded-full px-1.5 py-0.5 text-[10px]"
                    >
                      {group.items.length}
                    </span>
                  </div>
                  <div class="flex items-center font-mono text-[11px]">
                    <span class="text-muted-foreground mr-1 text-[10px]">Net:</span>
                    <AmountDisplay
                      amount={group.net}
                      currency={baseCurrency}
                      class="font-semibold {group.net >= 0 ? 'text-emerald-500' : 'text-rose-500'}"
                    />
                  </div>
                </div>
              </Table.Cell>
            </Table.Row>

            <!-- Transactions in this timeline group (omitted when group is collapsed) -->
            {#if !collapsedGroupKeys.has(group.dateKey)}
              {#each group.items as tx (tx.id)}
                <TransactionRow
                  {tx}
                  draft={rowDrafts[tx.id]}
                  isDirty={hasRowDraft(tx.id)}
                  {types}
                  {members}
                  {accounts}
                  {currencies}
                  onSave={onSaveRowDraft}
                  onDiscard={onDiscardRowDraft}
                  onDelete={onDeleteTransaction}
                  {onDraftChange}
                />
              {/each}
            {/if}
          {/each}
        {/if}
      </Table.Body>
    </Table.Root>
  </div>
</div>
