<script lang="ts">
  import { authStore } from "$lib/features/auth";
  import { AuthModal, MarketingHero } from "$components";
  import { page } from "$app/state";

  let showAuthModal = $state(false);

  $effect(() => {
    if (!authStore.isAuthenticated && page.url.searchParams.get("auth") === "login") {
      showAuthModal = true;
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
    <!-- Clean home canvas for authenticated users -->
  {:else}
    <!-- Marketing Hero Page for Unauthenticated Visitors -->
    <div class="mx-auto flex w-full max-w-6xl flex-1 flex-col p-6 sm:py-10">
      <MarketingHero onOpenAuth={() => (showAuthModal = true)} />
    </div>
  {/if}
</div>

<AuthModal isOpen={showAuthModal} onClose={() => (showAuthModal = false)} />
