<script lang="ts">
  import { LandmarkIcon, ChevronDownIcon, ChevronRightIcon, CheckIcon } from "@lucide/svelte";
  import * as Popover from "$lib/components/ui/popover";
  import type { Member, Account } from "$lib/features/family/types";
  import type { MemberId, AccountId } from "$lib/types";

  interface Props {
    members?: readonly Member[];
    accounts?: readonly Account[];
    selectedMemberIds?: MemberId[];
    selectedAccountIds?: AccountId[];
  }

  let {
    members = [],
    accounts = [],
    selectedMemberIds = $bindable<MemberId[]>([]),
    selectedAccountIds = $bindable<AccountId[]>([]),
  }: Props = $props();

  let menuMemberId = $state<MemberId | null>(null);

  const scopedMenuAccounts = $derived.by(() => {
    return menuMemberId ? accounts.filter((a) => a.ownerMemberId === menuMemberId) : accounts;
  });

  const isMemberAccountFiltered = $derived(
    selectedMemberIds.length > 0 || selectedAccountIds.length > 0,
  );

  function getAccountName(acc: Account): string {
    return acc.type === "bank_account" ? acc.accountName : acc.cardName;
  }

  function toggleMember(id: MemberId) {
    if (selectedMemberIds.includes(id)) {
      selectedMemberIds = selectedMemberIds.filter((m) => m !== id);
    } else {
      selectedMemberIds = [...selectedMemberIds, id];
    }
  }

  function memberOnly(id: MemberId) {
    selectedMemberIds = [id];
    selectedAccountIds = [];
  }

  function toggleAccount(id: AccountId) {
    if (selectedAccountIds.includes(id)) {
      selectedAccountIds = selectedAccountIds.filter((a) => a !== id);
    } else {
      selectedAccountIds = [...selectedAccountIds, id];
    }
  }

  function accountOnly(id: AccountId) {
    selectedAccountIds = [id];
  }

  function clearAccountFilters() {
    selectedMemberIds = [];
    selectedAccountIds = [];
  }
</script>

<Popover.Root>
  <Popover.Trigger
    class="inline-flex h-8.5 items-center gap-2 rounded-lg border px-3 text-xs font-medium whitespace-nowrap transition-colors {isMemberAccountFiltered
      ? 'border-foreground bg-foreground text-background font-semibold shadow-xs'
      : 'border-border/40 text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
  >
    <LandmarkIcon class="size-3.5" />
    <span>Account</span>
    <ChevronDownIcon class="size-3 opacity-60" />
  </Popover.Trigger>
  <Popover.Content
    side="bottom"
    align="start"
    sideOffset={6}
    class="border-border/60 bg-popover text-popover-foreground w-auto overflow-hidden rounded-xl border p-0 shadow-xl"
  >
    <div class="divide-border/40 flex h-72 divide-x">
      <!-- Left Layer: Members -->
      <div class="flex h-full w-40 shrink-0 flex-col">
        <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
          <!-- All Option -->
          <button
            type="button"
            onclick={() => {
              menuMemberId = null;
            }}
            class="flex w-full items-center justify-between rounded-lg p-2 text-sm transition-colors {menuMemberId ===
            null
              ? 'bg-muted text-foreground font-medium'
              : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
          >
            <span>All</span>
            <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
          </button>

          <!-- Individual Members -->
          {#each members as m (m.id)}
            {@const isChecked = selectedMemberIds.includes(m.id)}
            {@const isHovered = menuMemberId === m.id}
            <div
              role="button"
              tabindex="0"
              onmouseenter={() => (menuMemberId = m.id)}
              onclick={() => (menuMemberId = m.id)}
              onkeydown={(e) => {
                if (e.key === "Enter") menuMemberId = m.id;
              }}
              class="group flex w-full cursor-pointer items-center justify-between rounded-lg p-2 text-sm transition-colors {isHovered
                ? 'bg-muted text-foreground'
                : 'hover:bg-muted/50 text-muted-foreground hover:text-foreground'}"
            >
              <button
                type="button"
                onclick={(e) => {
                  e.stopPropagation();
                  toggleMember(m.id);
                }}
                class="flex flex-1 items-center gap-2.5 text-left"
              >
                <span
                  class="flex size-4 items-center justify-center rounded border {isChecked
                    ? 'border-foreground bg-foreground text-background'
                    : 'border-muted-foreground/40 bg-transparent'}"
                >
                  {#if isChecked}
                    <CheckIcon class="size-3 stroke-3" />
                  {/if}
                </span>
                <span class={isChecked ? "text-foreground font-medium" : ""}>{m.memberName}</span>
              </button>

              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onclick={(e) => {
                    e.stopPropagation();
                    memberOnly(m.id);
                  }}
                  class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[10px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                  title="Show only {m.memberName}"
                >
                  Only
                </button>
                <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
              </div>
            </div>
          {/each}
        </div>

        <!-- Footer Quick Filter -->
        <div class="border-border/40 border-t p-2">
          <button
            type="button"
            onclick={clearAccountFilters}
            class="hover:bg-muted text-muted-foreground hover:text-foreground w-full rounded-md py-1.5 text-center text-xs transition-colors"
          >
            Clear Account Filters
          </button>
        </div>
      </div>

      <!-- Right Layer: Accounts for Selected Member (or All Accounts) -->
      <div class="flex h-full w-56 flex-col">
        <div class="flex-1 space-y-1 overflow-y-auto p-1.5">
          {#if scopedMenuAccounts.length === 0}
            <div class="text-muted-foreground/60 p-4 text-center text-xs">No accounts found</div>
          {:else}
            {#each scopedMenuAccounts as acc (acc.id)}
              {@const isChecked = selectedAccountIds.includes(acc.id)}
              <div
                class="hover:bg-muted/50 group flex items-center justify-between rounded-lg p-2 text-sm transition-colors"
              >
                <button
                  type="button"
                  onclick={() => toggleAccount(acc.id)}
                  class="flex flex-1 items-center gap-2.5 text-left"
                >
                  <span
                    class="flex size-4 items-center justify-center rounded border {isChecked
                      ? 'border-foreground bg-foreground text-background'
                      : 'border-muted-foreground/40 bg-transparent'}"
                  >
                    {#if isChecked}
                      <CheckIcon class="size-3 stroke-3" />
                    {/if}
                  </span>
                  <div class="truncate">
                    <div class="truncate font-medium {isChecked ? 'text-foreground' : ''}">
                      {getAccountName(acc)}
                    </div>
                    <div class="text-muted-foreground font-mono text-[10px]">
                      {acc.bankName} ····{acc.last4}
                    </div>
                  </div>
                </button>

                <button
                  type="button"
                  onclick={() => accountOnly(acc.id)}
                  class="bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 font-mono text-[10px] tracking-wider uppercase opacity-0 transition-all group-hover:opacity-100"
                  title="Show only {getAccountName(acc)}"
                >
                  Only
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </Popover.Content>
</Popover.Root>
