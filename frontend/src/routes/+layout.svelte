<script lang="ts">
  import "../app.css";
  import { TopNav, AppSidebar } from "$components";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
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
  <Sidebar.Provider>
    <AppSidebar />
    <Sidebar.Inset>
      <header class="flex h-12 shrink-0 items-center border-b border-border/40 px-4">
        <div class="flex items-center gap-2">
          <Sidebar.Trigger class="-ml-1" />
        </div>
      </header>
      <div class="flex flex-1 flex-col overflow-y-auto">
        {@render children()}
      </div>
    </Sidebar.Inset>
  </Sidebar.Provider>
{:else}
  <div class="flex min-h-screen flex-col bg-background text-foreground">
    <TopNav />
    <main class="flex flex-1 flex-col">
      {@render children()}
    </main>
  </div>
{/if}
