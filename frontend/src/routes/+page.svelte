<script lang="ts">
  import { authStore } from "$lib/features/auth";
  import { AuthModal, MarketingHero } from "$components";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";

  let showAuthModal = $state(false);

  $effect(() => {
    if (authStore.isAuthenticated && authStore.currentUser) {
      void goto(resolve("/configuration/family"), { replaceState: true });
    }
  });
</script>

<svelte:head>
  <title>CoSave — Family Finance, Budgets & Savings</title>
</svelte:head>

<div class="flex flex-1 flex-col">
  {#if authStore.isLoading}
    <div class="flex flex-1 items-center justify-center py-20">
      <div
        class="size-8 animate-spin rounded-full border-2 border-(--border-subtle) border-t-emerald-500"
      ></div>
    </div>
  {:else if authStore.isAuthenticated && authStore.currentUser}
    <div class="flex flex-1 items-center justify-center py-20">
      <div
        class="border-border/40 size-8 animate-spin rounded-full border-2 border-t-emerald-500"
      ></div>
    </div>
  {:else}
    <!-- Marketing Hero Page for Unauthenticated Visitors -->
    <div class="mx-auto flex w-full max-w-6xl flex-1 flex-col p-6 sm:py-10">
      <MarketingHero onOpenAuth={() => (showAuthModal = true)} />
    </div>
  {/if}
</div>

<AuthModal isOpen={showAuthModal} onClose={() => (showAuthModal = false)} />
