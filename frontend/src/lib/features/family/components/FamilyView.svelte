<script lang="ts">
  import { familyStore } from "../store.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import LandmarkIcon from "@lucide/svelte/icons/landmark";
  import UserIcon from "@lucide/svelte/icons/user";
  import UsersIcon from "@lucide/svelte/icons/users";
  import AddMemberModal from "./AddMemberModal.svelte";
  import EditMemberModal from "./EditMemberModal.svelte";
  import AddAccountModal from "./AddAccountModal.svelte";
  import EditAccountModal from "./EditAccountModal.svelte";
  import ConfirmDeleteModal from "./ConfirmDeleteModal.svelte";
  import type { Account, AccountType, Member } from "../types";

  let isAddMemberOpen = $state(false);
  let isEditMemberOpen = $state(false);
  let isAddAccountOpen = $state(false);
  let isEditAccountOpen = $state(false);
  let selectedAccountForEdit = $state<Account | null>(null);
  let addAccountDefaultType = $state<AccountType>("bank_account");

  let isConfirmDeleteOpen = $state(false);
  let deleteConfirmTitle = $state("");
  let deleteConfirmDescription = $state("");
  let pendingDeleteAction = $state<(() => void) | null>(null);

  const activeMember = $derived(
    familyStore.getMember(familyStore.selectedMemberId) ?? familyStore.members[0],
  );

  const activeBankAccounts = $derived(
    activeMember ? familyStore.getMemberBankAccounts(activeMember.id) : [],
  );

  const activeCreditCards = $derived(
    activeMember ? familyStore.getMemberCreditCards(activeMember.id) : [],
  );

  function openAddAccount(type: AccountType) {
    addAccountDefaultType = type;
    isAddAccountOpen = true;
  }

  function openEditAccount(account: Account) {
    selectedAccountForEdit = account;
    isEditAccountOpen = true;
  }

  function promptDeleteMember(member: Member) {
    deleteConfirmTitle = `Delete ${member.name}?`;
    deleteConfirmDescription = `Are you sure you want to delete ${member.name} and all associated accounts? This action cannot be undone.`;
    pendingDeleteAction = () => familyStore.deleteMember(member.id);
    isConfirmDeleteOpen = true;
  }

  function promptDeleteAccount(account: Account) {
    const label =
      account.type === "bank_account"
        ? `${account.bankName} •••• ${account.last4}`
        : `${account.cardName} (${account.bankName} •••• ${account.last4})`;
    deleteConfirmTitle = "Delete Account?";
    deleteConfirmDescription = `Are you sure you want to delete ${label}? This action cannot be undone.`;
    pendingDeleteAction = () => familyStore.deleteAccount(account.id);
    isConfirmDeleteOpen = true;
  }
</script>

<div class="flex flex-col gap-6">
  <!-- Page Header -->
  <div
    class="border-border/40 flex flex-col justify-between gap-3 border-b pb-4 sm:flex-row sm:items-center"
  >
    <div>
      <h2 class="text-foreground text-xl font-bold tracking-tight">Family & Accounts</h2>
    </div>

    <div class="flex items-center gap-2">
      <Button
        size="sm"
        class="bg-foreground text-background hover:bg-foreground/90 font-medium"
        onclick={() => (isAddMemberOpen = true)}
      >
        <PlusIcon data-icon="inline-start" />
        Add Member
      </Button>
    </div>
  </div>

  <!-- Master-Detail Split Container -->
  <div class="grid grid-cols-1 gap-6 lg:grid-cols-12">
    <!-- Left Column: Master List of Members -->
    <div class="flex flex-col gap-3 lg:col-span-4">
      <div class="flex items-center justify-between px-1">
        <span class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
          Family Members ({familyStore.members.length})
        </span>
        <span class="text-muted-foreground font-mono text-xs">
          {familyStore.accounts.length} Accounts
        </span>
      </div>

      <div class="flex flex-col gap-2">
        {#each familyStore.members as member (member.id)}
          {@const bankCount = familyStore.getMemberBankAccounts(member.id).length}
          {@const cardCount = familyStore.getMemberCreditCards(member.id).length}
          {@const isSelected = activeMember?.id === member.id}

          <button
            type="button"
            class={`group relative flex w-full flex-col gap-2 rounded-xl border p-4 text-left transition-all ${
              isSelected
                ? "border-foreground/30 bg-muted/40 shadow-xs"
                : "border-border/40 bg-card hover:border-border hover:bg-muted/20"
            }`}
            onclick={() => (familyStore.selectedMemberId = member.id)}
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class={`flex size-9 items-center justify-center rounded-lg border text-sm font-semibold transition-colors ${
                    isSelected
                      ? "border-foreground/40 bg-foreground text-background"
                      : "border-border/60 bg-muted/30 text-foreground"
                  }`}
                >
                  {member.name.charAt(0).toUpperCase()}
                </div>
                <div class="flex flex-col">
                  <span class="text-foreground text-sm font-semibold tracking-tight">
                    {member.name}
                  </span>
                  <span class="text-muted-foreground text-xs">
                    {bankCount}
                    {bankCount === 1 ? "Bank Account" : "Bank Accounts"}, {cardCount}
                    {cardCount === 1 ? "Credit Card" : "Credit Cards"}
                  </span>
                </div>
              </div>
            </div>
          </button>
        {/each}

        {#if familyStore.members.length === 0}
          <div
            class="border-border/60 flex flex-col items-center justify-center rounded-xl border border-dashed p-8 text-center"
          >
            <UsersIcon class="text-muted-foreground/60 mb-2 size-8" />
            <p class="text-muted-foreground text-xs">No family members yet.</p>
            <Button
              variant="outline"
              size="sm"
              class="mt-3"
              onclick={() => (isAddMemberOpen = true)}
            >
              Add First Member
            </Button>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right Column: Unified Detail Workspace for Active Member -->
    <div class="flex flex-col lg:col-span-8">
      {#if activeMember}
        <!-- Single Unified Surface Container -->
        <div class="border-border/40 bg-card flex flex-col gap-6 rounded-2xl border p-6">
          <!-- Member Profile Header -->
          <div
            class="border-border/30 flex flex-col justify-between gap-4 border-b pb-5 sm:flex-row sm:items-center"
          >
            <div class="flex items-center gap-3.5">
              <div
                class="border-border/60 bg-muted/40 text-foreground flex size-11 items-center justify-center rounded-xl border text-lg font-bold"
              >
                <UserIcon class="size-5" />
              </div>
              <div class="flex flex-col">
                <h3 class="text-foreground text-lg font-bold tracking-tight">
                  {activeMember.name}
                </h3>
                <p class="text-muted-foreground text-xs">
                  {activeBankAccounts.length}
                  {activeBankAccounts.length === 1 ? "Bank Account" : "Bank Accounts"} &bull; {activeCreditCards.length}
                  {activeCreditCards.length === 1 ? "Credit Card" : "Credit Cards"}
                </p>
              </div>
            </div>

            <!-- Member Action Icons (Edit & Delete) -->
            <div class="flex items-center gap-1.5">
              <Button
                variant="outline"
                size="sm"
                class="border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted size-8 p-0"
                title="Edit member name"
                aria-label="Edit member"
                onclick={() => (isEditMemberOpen = true)}
              >
                <PencilIcon class="size-3.5" />
                <span class="sr-only">Edit Member</span>
              </Button>
              <Button
                variant="outline"
                size="sm"
                class="border-border/40 text-destructive hover:bg-destructive/10 hover:text-destructive size-8 p-0"
                title="Delete member"
                aria-label="Delete member"
                onclick={() => promptDeleteMember(activeMember)}
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
                <Badge variant="secondary" class="font-mono text-[11px]">
                  {activeBankAccounts.length}
                </Badge>
              </div>

              <Button
                variant="outline"
                size="sm"
                class="border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted size-8 p-0"
                title="Add Bank Account"
                aria-label="Add Bank Account"
                onclick={() => openAddAccount("bank_account")}
              >
                <PlusIcon class="size-3.5" />
                <span class="sr-only">Add Bank Account</span>
              </Button>
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {#each activeBankAccounts as acc (acc.id)}
                <div
                  class="border-border/30 bg-muted/20 hover:border-border/60 hover:bg-muted/30 flex items-center justify-between gap-3 rounded-xl border p-4 transition-all"
                >
                  <div class="flex min-w-0 flex-1 items-center gap-3">
                    <div
                      class="border-border/40 bg-background text-muted-foreground flex size-9 shrink-0 items-center justify-center rounded-lg border"
                    >
                      <LandmarkIcon class="size-4" />
                    </div>
                    <div class="flex min-w-0 flex-1 flex-col">
                      <span class="text-foreground truncate text-sm font-semibold"
                        >{acc.bankName}</span
                      >
                      <span class="text-muted-foreground font-mono text-xs">
                        &bull;&bull;&bull;&bull; {acc.last4}
                      </span>
                    </div>
                  </div>

                  <div class="flex shrink-0 items-center gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      class="text-muted-foreground hover:text-foreground size-8 p-0"
                      title="Edit account"
                      aria-label="Edit account"
                      onclick={() => openEditAccount(acc)}
                    >
                      <PencilIcon class="size-3.5" />
                      <span class="sr-only">Edit account</span>
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      class="text-destructive hover:text-destructive hover:bg-destructive/10 size-8 p-0"
                      title="Delete account"
                      aria-label="Delete account"
                      onclick={() => promptDeleteAccount(acc)}
                    >
                      <Trash2Icon class="size-3.5" />
                      <span class="sr-only">Delete account</span>
                    </Button>
                  </div>
                </div>
              {/each}

              {#if activeBankAccounts.length === 0}
                <div
                  class="border-border/40 bg-muted/10 col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed p-6 text-center"
                >
                  <p class="text-muted-foreground text-xs">
                    No bank accounts linked to {activeMember.name}.
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    class="mt-2.5"
                    onclick={() => openAddAccount("bank_account")}
                  >
                    <PlusIcon data-icon="inline-start" />
                    Link Bank Account
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
                <Badge variant="secondary" class="font-mono text-[11px]">
                  {activeCreditCards.length}
                </Badge>
              </div>

              <Button
                variant="outline"
                size="sm"
                class="border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted size-8 p-0"
                title="Add Credit Card"
                aria-label="Add Credit Card"
                onclick={() => openAddAccount("credit_card")}
              >
                <PlusIcon class="size-3.5" />
                <span class="sr-only">Add Credit Card</span>
              </Button>
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {#each activeCreditCards as card (card.id)}
                <div
                  class="border-border/30 bg-muted/20 hover:border-border/60 hover:bg-muted/30 flex flex-col justify-between gap-3 rounded-xl border p-4 transition-all"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex min-w-0 flex-1 items-center gap-3">
                      <div
                        class="border-border/40 bg-background text-foreground flex size-9 shrink-0 items-center justify-center rounded-lg border"
                      >
                        <CreditCardIcon class="size-4" />
                      </div>
                      <div class="flex min-w-0 flex-1 flex-col">
                        <span class="text-foreground truncate text-sm leading-tight font-semibold">
                          {card.cardName}
                        </span>
                        <span class="text-muted-foreground truncate text-xs">
                          {card.bankName} &bull;
                          <span class="font-mono">&bull;&bull;&bull;&bull; {card.last4}</span>
                        </span>
                      </div>
                    </div>

                    <div class="flex shrink-0 items-center gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        class="text-muted-foreground hover:text-foreground size-8 p-0"
                        title="Edit card"
                        aria-label="Edit card"
                        onclick={() => openEditAccount(card)}
                      >
                        <PencilIcon class="size-3.5" />
                        <span class="sr-only">Edit card</span>
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        class="text-destructive hover:text-destructive hover:bg-destructive/10 size-8 p-0"
                        title="Delete card"
                        aria-label="Delete card"
                        onclick={() => promptDeleteAccount(card)}
                      >
                        <Trash2Icon class="size-3.5" />
                        <span class="sr-only">Delete card</span>
                      </Button>
                    </div>
                  </div>

                  <div class="border-border/20 flex items-center justify-between border-t pt-2.5">
                    <span
                      class="text-muted-foreground text-[11px] font-semibold tracking-wider uppercase"
                    >
                      Credit Limit
                    </span>
                    <span class="text-foreground font-mono text-xs font-bold">
                      ${card.creditLimit.toLocaleString()}
                    </span>
                  </div>
                </div>
              {/each}

              {#if activeCreditCards.length === 0}
                <div
                  class="border-border/40 bg-muted/10 col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed p-6 text-center"
                >
                  <p class="text-muted-foreground text-xs">
                    No credit cards linked to {activeMember.name}.
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    class="mt-2.5"
                    onclick={() => openAddAccount("credit_card")}
                  >
                    <PlusIcon data-icon="inline-start" />
                    Link Credit Card
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
  </div>
</div>

<AddMemberModal open={isAddMemberOpen} onClose={() => (isAddMemberOpen = false)} />
<EditMemberModal
  open={isEditMemberOpen}
  member={activeMember ?? null}
  onClose={() => (isEditMemberOpen = false)}
/>
<AddAccountModal
  open={isAddAccountOpen}
  defaultMemberId={activeMember?.id}
  defaultType={addAccountDefaultType}
  onClose={() => (isAddAccountOpen = false)}
/>
<EditAccountModal
  open={isEditAccountOpen}
  account={selectedAccountForEdit}
  onClose={() => {
    isEditAccountOpen = false;
    selectedAccountForEdit = null;
  }}
/>
<ConfirmDeleteModal
  open={isConfirmDeleteOpen}
  title={deleteConfirmTitle}
  description={deleteConfirmDescription}
  onConfirm={() => pendingDeleteAction?.()}
  onClose={() => (isConfirmDeleteOpen = false)}
/>
