<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";

  import { ThemeSelector } from "$lib/components";

  import { authStore } from "$lib/features/auth";

  const user = $derived(authStore.requireUser());
</script>

<svelte:head>
  <title>CoSave — Settings</title>
</svelte:head>

<div class="mx-auto w-full max-w-3xl flex-1 space-y-4 py-6">
  <h1 class="text-xl font-semibold tracking-tight text-(--text-primary)">Settings</h1>

  <div
    class="flex w-full flex-col justify-between gap-4 rounded-xl border border-(--border-subtle) bg-(--bg-surface) p-4 sm:flex-row sm:items-center sm:p-5"
  >
    <div class="flex items-center gap-3">
      <div
        class="flex size-9 items-center justify-center rounded-lg bg-emerald-500/20 text-xs font-bold text-emerald-400"
      >
        {user.username.slice(0, 2).toUpperCase()}
      </div>
      <div class="flex items-center gap-2">
        <span class="text-sm font-semibold text-(--text-primary)">{user.username}</span>
        <Badge variant={user.role === "admin" ? "default" : "secondary"}>
          {user.role}
        </Badge>
      </div>
    </div>

    <Button variant="destructive" size="sm" onclick={() => authStore.logout()}>Sign Out</Button>
  </div>

  <ThemeSelector />
</div>
