<script lang="ts">
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import LandmarkIcon from "@lucide/svelte/icons/landmark";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  import { Button } from "$lib/components/ui/button";

  import AmountDisplay from "$lib/components/AmountDisplay.svelte";

  import type { Currency } from "$lib/types";
  import type { Account } from "../types";

  interface Props {
    account: Account;
    currency: Currency;
    onEdit: (account: Account) => void;
    onDelete: (account: Account) => void;
  }

  const { account, currency, onEdit, onDelete }: Props = $props();
</script>

<div
  class="border-border/30 bg-muted/20 hover:border-border/60 hover:bg-muted/30 flex flex-col justify-between gap-3 rounded-xl border p-4 transition-all"
>
  <div class="flex items-start justify-between gap-3">
    <div class="flex min-w-0 flex-1 items-center gap-3">
      <div
        class="border-border/40 bg-background flex size-9 shrink-0 items-center justify-center rounded-lg border {account.type ===
        'credit_card'
          ? 'text-foreground'
          : 'text-muted-foreground'}"
      >
        {#if account.type === "bank_account"}
          <LandmarkIcon class="size-4" />
        {:else}
          <CreditCardIcon class="size-4" />
        {/if}
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <span class="text-foreground truncate text-sm font-semibold">
          {account.type === "bank_account" ? account.accountName : account.cardName}
        </span>
        <span class="text-muted-foreground truncate text-xs">
          {account.bankName} &bull;
          <span class="font-mono">&bull;&bull;&bull;&bull; {account.last4}</span>
        </span>
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        class="text-muted-foreground hover:text-foreground size-8 p-0"
        title={account.type === "bank_account" ? "Edit account" : "Edit card"}
        aria-label={account.type === "bank_account" ? "Edit account" : "Edit card"}
        onclick={() => onEdit(account)}
      >
        <PencilIcon class="size-3.5" />
        <span class="sr-only">
          {account.type === "bank_account" ? "Edit account" : "Edit card"}
        </span>
      </Button>
      <Button
        variant="ghost"
        size="sm"
        class="text-destructive hover:text-destructive hover:bg-destructive/10 size-8 p-0"
        title={account.type === "bank_account" ? "Delete account" : "Delete card"}
        aria-label={account.type === "bank_account" ? "Delete account" : "Delete card"}
        onclick={() => onDelete(account)}
      >
        <Trash2Icon class="size-3.5" />
        <span class="sr-only">
          {account.type === "bank_account" ? "Delete account" : "Delete card"}
        </span>
      </Button>
    </div>
  </div>

  {#if account.type === "bank_account"}
    <!-- Bank Account Available Balance -->
    <div class="border-border/20 flex items-center justify-between border-t pt-2.5">
      <span class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
        Available Balance
      </span>
      <AmountDisplay amount={account.availableBalance} {currency} class="text-xs font-bold" />
    </div>
  {:else}
    <!-- Credit Card Limit, Available & Outstanding Metrics -->
    <div class="border-border/20 grid grid-cols-3 gap-2 border-t pt-2.5 text-center sm:text-left">
      <div class="flex flex-col">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">
          Limit
        </span>
        <AmountDisplay amount={account.creditLimit} {currency} class="text-xs font-semibold" />
      </div>

      <div class="flex flex-col">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">
          Available
        </span>
        <AmountDisplay
          amount={account.availableCredit}
          {currency}
          class="text-xs font-semibold text-emerald-600 dark:text-emerald-400"
        />
      </div>

      <div class="flex flex-col">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">
          Outstanding
        </span>
        <AmountDisplay amount={account.outstandingBalance} {currency} class="text-xs font-bold" />
      </div>
    </div>
  {/if}
</div>
