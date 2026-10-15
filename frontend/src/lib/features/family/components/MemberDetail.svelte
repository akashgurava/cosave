<script lang="ts">
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import LandmarkIcon from "@lucide/svelte/icons/landmark";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import UserIcon from "@lucide/svelte/icons/user";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";

  import type { Currency } from "$lib/types";
  import type { Account, AccountType, BankAccount, CreditCardAccount, Member } from "../types";

  import AccountCard from "./AccountCard.svelte";

  interface Props {
    member: Member | null;
    bankAccounts: readonly BankAccount[];
    creditCards: readonly CreditCardAccount[];
    currency: Currency | null;
    onEditMember: () => void;
    onDeleteMember: () => void;
    onAddAccount: (type: AccountType) => void;
    onEditAccount: (account: Account) => void;
    onDeleteAccount: (account: Account) => void;
  }

  const {
    member,
    bankAccounts,
    creditCards,
    currency,
    onEditMember,
    onDeleteMember,
    onAddAccount,
    onEditAccount,
    onDeleteAccount,
  }: Props = $props();
</script>

<div class="flex flex-col">
  {#if member !== null && currency !== null}
    <!-- Single Unified Surface Container -->
    <div class="bg-card flex flex-col gap-6 rounded-2xl border border-(--border-subtle) p-6">
      <!-- Member Profile Header -->
      <div class="flex items-center justify-between gap-4 border-b border-(--border-subtle) pb-5">
        <div class="flex items-center gap-3.5">
          <div
            class="bg-muted/40 text-foreground flex size-11 items-center justify-center rounded-xl border border-(--border-subtle) text-lg font-bold"
          >
            <UserIcon class="size-5" />
          </div>
          <h3 class="text-foreground text-lg font-bold tracking-tight">
            {member.name}
          </h3>
        </div>

        <!-- Member Action Buttons -->
        <div class="flex items-center gap-1.5">
          <Button
            variant="outline"
            size="sm"
            class="text-muted-foreground hover:text-foreground hover:bg-muted size-8 border-(--border-subtle) p-0"
            title="Edit member name"
            aria-label="Edit member"
            onclick={onEditMember}
          >
            <PencilIcon class="size-3.5" />
            <span class="sr-only">Edit Member</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            class="text-destructive hover:bg-destructive/10 hover:text-destructive size-8 border-(--border-subtle) p-0"
            title="Delete member"
            aria-label="Delete member"
            onclick={onDeleteMember}
          >
            <Trash2Icon class="size-3.5" />
            <span class="sr-only">Delete Member</span>
          </Button>
        </div>
      </div>

      <!-- Bank Accounts Section -->
      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <LandmarkIcon class="text-muted-foreground size-4" />
            <h4 class="text-foreground text-sm font-semibold tracking-tight">Bank Accounts</h4>
            <Badge variant="secondary" class="font-mono text-xs">
              {bankAccounts.length}
            </Badge>
          </div>

          <Button
            variant="outline"
            size="sm"
            class="border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted size-8 p-0"
            title="Add Bank Account"
            aria-label="Add Bank Account"
            onclick={() => onAddAccount("bank_account")}
          >
            <PlusIcon class="size-3.5" />
            <span class="sr-only">Add Bank Account</span>
          </Button>
        </div>

        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {#each bankAccounts as acc (acc.id)}
            <AccountCard
              account={acc}
              {currency}
              onEdit={onEditAccount}
              onDelete={onDeleteAccount}
            />
          {/each}

          {#if bankAccounts.length === 0}
            <div
              class="border-border/40 bg-muted/10 col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed p-6 text-center"
            >
              <p class="text-muted-foreground text-xs">
                No bank accounts added for {member.name}.
              </p>
              <Button
                variant="outline"
                size="sm"
                class="mt-2.5"
                onclick={() => onAddAccount("bank_account")}
              >
                <PlusIcon data-icon="inline-start" />
                Add Bank Account
              </Button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Credit Cards Section -->
      <div class="flex flex-col gap-3 pt-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <CreditCardIcon class="text-muted-foreground size-4" />
            <h4 class="text-foreground text-sm font-semibold tracking-tight">Credit Cards</h4>
            <Badge variant="secondary" class="font-mono text-xs">
              {creditCards.length}
            </Badge>
          </div>

          <Button
            variant="outline"
            size="sm"
            class="border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted size-8 p-0"
            title="Add Credit Card"
            aria-label="Add Credit Card"
            onclick={() => onAddAccount("credit_card")}
          >
            <PlusIcon class="size-3.5" />
            <span class="sr-only">Add Credit Card</span>
          </Button>
        </div>

        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {#each creditCards as card (card.id)}
            <AccountCard
              account={card}
              {currency}
              onEdit={onEditAccount}
              onDelete={onDeleteAccount}
            />
          {/each}

          {#if creditCards.length === 0}
            <div
              class="border-border/40 bg-muted/10 col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed p-6 text-center"
            >
              <p class="text-muted-foreground text-xs">
                No credit cards added for {member.name}.
              </p>
              <Button
                variant="outline"
                size="sm"
                class="mt-2.5"
                onclick={() => onAddAccount("credit_card")}
              >
                <PlusIcon data-icon="inline-start" />
                Add Credit Card
              </Button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <div
      class="border-border/40 bg-card flex flex-col items-center justify-center rounded-2xl border p-12 text-center"
    >
      <p class="text-muted-foreground text-sm">Select a member to view their accounts.</p>
    </div>
  {/if}
</div>
