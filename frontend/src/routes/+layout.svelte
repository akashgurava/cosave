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

  const SIDEBAR_STORAGE_KEY = "cosave_sidebar_open";
  let sidebarOpen = $state<boolean>(true);

  if (typeof window !== "undefined") {
    const saved = localStorage.getItem(SIDEBAR_STORAGE_KEY);
    if (saved !== null) {
      sidebarOpen = saved === "true";
    }
  }

  function handleSidebarOpenChange(open: boolean) {
    sidebarOpen = open;
    if (typeof window !== "undefined") {
      localStorage.setItem(SIDEBAR_STORAGE_KEY, String(open));
    }
  }

  $effect(() => {
    themeStore.applyTheme();
  });

  $effect(() => {
    return healthStore.startPolling(30000);
  });
</script>

{#if authStore.isAuthenticated}
  <Sidebar.Provider
    bind:open={sidebarOpen}
    onOpenChange={handleSidebarOpenChange}
    style="--sidebar-width-icon: 4rem;"
  >
    <AppSidebar />
    <Sidebar.Inset>
      <main class="flex flex-1 flex-col overflow-y-auto p-4 md:p-6 lg:p-8">
        {@render children()}
      </main>
    </Sidebar.Inset>
  </Sidebar.Provider>
{:else}
  <div class="bg-background text-foreground flex min-h-screen flex-col">
    <TopNav />
    <main class="flex flex-1 flex-col">
      {@render children()}
    </main>
  </div>
{/if}
