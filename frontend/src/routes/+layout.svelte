<script lang="ts">
  import "../app.css";
  import { TopNav, Sidebar } from "$components";
  import { authStore } from "$lib/auth";
  import { healthStore } from "$lib/health";
  import { themeStore } from "$lib/theme";
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  $effect(() => {
    themeStore.applyTheme();
  });

  $effect(() => {
    return healthStore.startPolling(30000);
  });
</script>

{#if authStore.isAuthenticated}
  <div class="flex min-h-screen flex-col bg-(--bg-primary) text-(--text-primary) md:flex-row">
    <Sidebar />
    <main class="flex min-h-screen flex-1 flex-col overflow-y-auto">
      {@render children()}
    </main>
  </div>
{:else}
  <div class="flex min-h-screen flex-col bg-(--bg-primary) text-(--text-primary)">
    <TopNav />
    <main class="flex flex-1 flex-col">
      {@render children()}
    </main>
  </div>
{/if}
