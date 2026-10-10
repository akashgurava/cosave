<script lang="ts">
  import { CheckIcon } from "@lucide/svelte";
  import * as Popover from "$lib/components/ui/popover";
  import type { Transaction } from "../../types";
  import type { Member, Account } from "$lib/features/family/types";
  import type { TransactionId } from "$lib/types";

  interface Props {
    txId: TransactionId;
    effectiveTx: Transaction;
    member: Member;
    account: Account;
    members: readonly Member[];
    accounts: readonly Account[];
    onDraftChange: (id: TransactionId, updates: Partial<Transaction>) => void;
  }

  let {
    txId,
    effectiveTx,
    member,
    account,
    members,
    accounts,
    onDraftChange,
  }: Props = $props();

  let open = $state(false);
</script>

<Popover.Root bind:open>
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
                  onDraftChange(txId, {
                    accountId: acc.id,
                  });
                  open = false;
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
                {#if effectiveTx.accountId === acc.id}
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
