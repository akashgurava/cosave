<script lang="ts">
  import { ThemeSelector } from "$components";
</script>

<svelte:head>
  <title>CoSave — Settings</title>
</svelte:head>

<div class="mx-auto flex w-full max-w-6xl flex-1 px-6 py-12">
  <div class="w-full max-w-3xl space-y-8">
    <div>
      <h1 class="text-xl font-semibold tracking-tight text-(--text-primary)">Settings</h1>
      <p class="mt-1 text-xs text-(--text-secondary)">
        Manage your workspace preferences and appearance.
      </p>
    </div>

    <ThemeSelector />
  </div>
</div>
