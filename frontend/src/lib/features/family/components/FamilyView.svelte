<script lang="ts">
  import { onMount } from "svelte";

  import PlusIcon from "@lucide/svelte/icons/plus";

  import { Button } from "$lib/components/ui/button";

  import { familyStore } from "../store.svelte";
  import type { Account, AccountType, Member } from "../types";

  import AccountModal from "./AccountModal.svelte";
  import ConfirmDeleteModal from "./ConfirmDeleteModal.svelte";
  import MemberDetail from "./MemberDetail.svelte";
  import MemberList from "./MemberList.svelte";
  import MemberModal from "./MemberModal.svelte";

  let isAddMemberOpen = $state(false);
  let isEditMemberOpen = $state(false);
  let isAddAccountOpen = $state(false);
  let isEditAccountOpen = $state(false);
  let selectedAccountForEdit = $state<Account | null>(null);
  let addAccountDefaultType = $state<AccountType>("bank_account");

  let isConfirmDeleteOpen = $state(false);
  let deleteConfirmTitle = $state("");
  let deleteConfirmDescription = $state("");
  let pendingDeleteAction = $state<(() => void | Promise<void>) | null>(null);

  onMount(async () => {
    await familyStore.load();
  });

  const activeMember = $derived.by(() => {
    const selected = familyStore.getMember(familyStore.selectedMemberId);
    if (selected !== null) return selected;
    const first = familyStore.members[0];
    return first !== undefined ? first : null;
  });

  const activeBankAccounts = $derived(
    activeMember !== null && activeMember !== undefined
      ? familyStore.getMemberBankAccounts(activeMember.id)
      : [],
  );

  const activeCreditCards = $derived(
    activeMember !== null && activeMember !== undefined
      ? familyStore.getMemberCreditCards(activeMember.id)
      : [],
  );

  const currency = $derived(familyStore.isSuccess === true ? familyStore.requireCurrency() : null);

  function openAddAccount(type: AccountType) {
    addAccountDefaultType = type;
    isAddAccountOpen = true;
  }

  function openEditAccount(account: Account) {
    selectedAccountForEdit = account;
    isEditAccountOpen = true;
  }

  function promptDeleteMember(member: Member) {
    deleteConfirmTitle = `Delete ${member.memberName}?`;
    deleteConfirmDescription = `Are you sure you want to delete ${member.memberName} and all associated accounts? This action cannot be undone.`;
    pendingDeleteAction = () => familyStore.deleteMember(member.id);
    isConfirmDeleteOpen = true;
  }

  function promptDeleteAccount(account: Account) {
    deleteConfirmTitle = "Delete Account?";
    deleteConfirmDescription = `Are you sure you want to delete "${account.name}"? This action cannot be undone.`;
    pendingDeleteAction = () => familyStore.deleteAccount(account.id);
    isConfirmDeleteOpen = true;
  }
</script>

<div class="flex flex-col gap-6">
  <!-- Page Header -->
  <div
    class="border-border/40 flex flex-col justify-between gap-3 border-b pb-4 sm:flex-row sm:items-center"
  >
    <div class="flex flex-wrap items-center gap-3">
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

  <!-- Global Error Banner with Retry -->
  {#if familyStore.error !== null}
    <div
      class="border-destructive/30 bg-destructive/10 text-destructive flex items-center justify-between rounded-xl border p-4 text-xs font-medium"
      role="alert"
    >
      <div class="flex items-center gap-2.5">
        <svg
          class="size-4 shrink-0"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{familyStore.error}</span>
      </div>
      <Button
        variant="outline"
        size="sm"
        class="border-destructive/40 text-destructive hover:bg-destructive/10 h-7 px-2.5 text-xs font-semibold"
        onclick={() => familyStore.load()}
      >
        Retry
      </Button>
    </div>
  {/if}

  <!-- Master-Detail Split Container -->
  <div class="grid grid-cols-1 gap-6 lg:grid-cols-12">
    <!-- Left Column: Master List of Members -->
    <div class="lg:col-span-4">
      <MemberList
        members={familyStore.members}
        totalAccountsCount={familyStore.accounts.length}
        isSelected={(id) => activeMember !== null && activeMember.id === id}
        getBankCount={(id) => familyStore.getMemberBankAccounts(id).length}
        getCardCount={(id) => familyStore.getMemberCreditCards(id).length}
        onSelect={(id) => (familyStore.selectedMemberId = id)}
        onAddMember={() => (isAddMemberOpen = true)}
      />
    </div>

    <!-- Right Column: Detail Workspace for Active Member -->
    <div class="lg:col-span-8">
      <MemberDetail
        member={activeMember}
        bankAccounts={activeBankAccounts}
        creditCards={activeCreditCards}
        {currency}
        onEditMember={() => (isEditMemberOpen = true)}
        onDeleteMember={() => {
          if (activeMember !== null) {
            promptDeleteMember(activeMember);
          }
        }}
        onAddAccount={openAddAccount}
        onEditAccount={openEditAccount}
        onDeleteAccount={promptDeleteAccount}
      />
    </div>
  </div>
</div>

<!-- Modals -->
<MemberModal bind:open={isAddMemberOpen} onClose={() => (isAddMemberOpen = false)} />
<MemberModal
  bind:open={isEditMemberOpen}
  member={activeMember}
  onClose={() => (isEditMemberOpen = false)}
/>
<AccountModal
  open={isAddAccountOpen}
  defaultMemberId={activeMember !== null ? activeMember.id : undefined}
  defaultType={addAccountDefaultType}
  onClose={() => (isAddAccountOpen = false)}
/>
<AccountModal
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
  onConfirm={async () => {
    if (pendingDeleteAction !== null) {
      await pendingDeleteAction();
    }
  }}
  onClose={() => (isConfirmDeleteOpen = false)}
/>
