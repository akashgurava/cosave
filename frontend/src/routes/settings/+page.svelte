<script lang="ts">
  import { authStore } from "$lib/auth";
  import { ThemeSelector } from "$components";
  import { Button, Badge, Card, CardHeader, CardTitle, CardContent } from "$components/ui";
</script>

<svelte:head>
  <title>CoSave — Settings</title>
</svelte:head>

<div class="mx-auto flex w-full max-w-6xl flex-1 px-6 py-12">
  <div class="w-full max-w-3xl space-y-8">
    <div>
      <h1 class="text-xl font-semibold tracking-tight text-(--text-primary)">Settings</h1>
      <p class="mt-1 text-xs text-(--text-secondary)">
        Manage your workspace preferences, appearance, and profile.
      </p>
    </div>

    {#if authStore.isAuthenticated && authStore.currentUser}
      {@const user = authStore.currentUser}
      <Card>
        <CardHeader class="pb-2">
          <CardTitle>Account Profile</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div
                class="flex size-10 items-center justify-center rounded-xl bg-emerald-500/20 text-sm font-bold text-emerald-400"
              >
                {user.name.slice(0, 2).toUpperCase()}
              </div>
              <div>
                <p class="text-xs font-semibold text-(--text-primary)">{user.name}</p>
              </div>
            </div>

            <div class="flex items-center gap-2.5">
              <Badge variant={user.role === "admin" ? "warning" : "secondary"}>
                {user.role}
              </Badge>
              <Button variant="destructive" size="sm" onclick={() => authStore.logout()}>
                Sign Out
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    {/if}

    <ThemeSelector />
  </div>
</div>
