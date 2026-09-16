<script lang="ts">
  import { authStore } from "$lib/auth";
  import { AuthModal, MarketingHero } from "$components";
  import { Badge, Card, CardContent, CardHeader, CardTitle, CardDescription } from "$components/ui";

  let showAuthModal = $state(false);
</script>

<svelte:head>
  <title>CoSave — Family Finance, Budgets & Savings</title>
</svelte:head>

<div class="mx-auto flex w-full max-w-6xl flex-1 flex-col p-6 sm:py-10">
  {#if authStore.isLoading}
    <div class="flex flex-1 items-center justify-center py-20">
      <div
        class="size-8 animate-spin rounded-full border-2 border-(--border-subtle) border-t-emerald-500"
      ></div>
    </div>
  {:else if authStore.isAuthenticated && authStore.currentUser}
    {@const user = authStore.currentUser}
    <!-- Authenticated Dashboard Header -->
    <div
      class="flex flex-col justify-between gap-4 border-b border-(--border-subtle) pb-8 sm:flex-row sm:items-center"
    >
      <div>
        <div class="flex items-center gap-3">
          <h1 class="text-2xl font-bold tracking-tight text-(--text-primary)">
            Welcome back, {user.name}
          </h1>
          <Badge variant={user.role === "admin" ? "warning" : "secondary"}>
            {user.role}
          </Badge>
        </div>
        <p class="mt-1 text-xs text-(--text-secondary)">Family Finance, Spend Tracking & Savings</p>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-xs text-(--text-muted)">
          Member since {new Date(user.created_at * 1000).toLocaleDateString()}
        </span>
      </div>
    </div>

    <!-- Quick Stats Grid -->
    <div class="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
      <Card>
        <CardHeader class="pb-2">
          <CardDescription>Total Family Balance</CardDescription>
          <CardTitle class="text-2xl font-bold">$0.00</CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-[11px] text-(--text-muted)">Ready for account linking</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader class="pb-2">
          <CardDescription>Monthly Budget & Expenses</CardDescription>
          <CardTitle class="text-2xl font-bold">$0.00</CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-[11px] text-(--text-muted)">0 active budgets configured</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader class="pb-2">
          <CardDescription>Family Savings</CardDescription>
          <CardTitle class="text-2xl font-bold">$0.00</CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-[11px] text-(--text-muted)">Track family savings and goals</p>
        </CardContent>
      </Card>
    </div>
  {:else}
    <!-- Marketing Hero Page for Unauthenticated Visitors -->
    <MarketingHero onOpenAuth={() => (showAuthModal = true)} />
  {/if}
</div>

<AuthModal isOpen={showAuthModal} onClose={() => (showAuthModal = false)} />
