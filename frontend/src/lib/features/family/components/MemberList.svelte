<script lang="ts">
  import UsersIcon from "@lucide/svelte/icons/users";

  import { Button } from "$lib/components/ui/button";

  import type { Member, MemberId } from "../types";

  interface Props {
    members: readonly Member[];
    totalAccountsCount: number;
    isSelected: (memberId: MemberId) => boolean;
    getBankCount: (memberId: MemberId) => number;
    getCardCount: (memberId: MemberId) => number;
    onSelect: (memberId: MemberId) => void;
    onAddMember: () => void;
  }

  const {
    members,
    totalAccountsCount,
    isSelected,
    getBankCount,
    getCardCount,
    onSelect,
    onAddMember,
  }: Props = $props();
</script>

<div class="flex flex-col gap-3">
  <!-- Section Header with Counts -->
  <div class="flex items-center justify-between px-1">
    <span class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
      Members ({members.length})
    </span>
    <span class="text-muted-foreground font-mono text-xs">
      {totalAccountsCount} Accounts
    </span>
  </div>

  <!-- Member Cards List -->
  <div class="flex flex-col gap-2">
    {#each members as member (member.id)}
      {@const bankCount = getBankCount(member.id)}
      {@const cardCount = getCardCount(member.id)}
      {@const selected = isSelected(member.id)}

      <button
        type="button"
        class="group relative flex w-full flex-col gap-2 rounded-xl border p-4 text-left transition-all {selected ===
        true
          ? 'border-foreground bg-foreground text-background shadow-xs'
          : 'border-border/40 bg-card text-foreground hover:border-border hover:bg-muted/20'}"
        onclick={() => onSelect(member.id)}
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div
              class="flex size-9 items-center justify-center rounded-lg border text-sm font-semibold transition-colors {selected ===
              true
                ? 'border-background/30 bg-background text-foreground'
                : 'border-border/60 bg-muted/30 text-foreground'}"
            >
              {member.memberName.charAt(0).toUpperCase()}
            </div>
            <div class="flex flex-col">
              <span
                class="text-sm font-semibold tracking-tight {selected === true
                  ? 'text-background'
                  : 'text-foreground'}"
              >
                {member.memberName}
              </span>
              <span
                class="text-xs {selected === true ? 'text-background/70' : 'text-muted-foreground'}"
              >
                {bankCount} bank &bull; {cardCount} credit
              </span>
            </div>
          </div>
        </div>
      </button>
    {/each}

    {#if members.length === 0}
      <div
        class="border-border/60 flex flex-col items-center justify-center rounded-xl border border-dashed p-8 text-center"
      >
        <UsersIcon class="text-muted-foreground/60 mb-2 size-8" />
        <p class="text-muted-foreground text-xs">No family members yet.</p>
        <Button variant="outline" size="sm" class="mt-3" onclick={onAddMember}>
          Add First Member
        </Button>
      </div>
    {/if}
  </div>
</div>
