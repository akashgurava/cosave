<script lang="ts">
  import "../app.css";
  import { TopNav } from "$components";
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

<div class="flex min-h-screen flex-col bg-(--bg-primary) text-(--text-primary)">
  <TopNav />
  <main class="flex flex-1 flex-col">
    {@render children()}
  </main>
</div>
