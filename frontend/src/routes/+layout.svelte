<script lang="ts">
  import "../app.css";
  import { TopNav, AppSidebar } from "$components";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator";
  import { authStore, AuthModal } from "$lib/features/auth";
  import { categoryStore } from "$lib/features/categories";
  import { familyStore } from "$lib/features/family";
  import { healthStore } from "$lib/health.svelte";
  import { themeStore } from "$lib/theme.svelte";
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  const SIDEBAR_STORAGE_KEY = "cosave_sidebar_open";
  let sidebarOpen = $state<boolean>(true);

  const PUBLIC_ROUTES = [resolve("/")];
  const isPublicRoute = $derived(PUBLIC_ROUTES.includes(page.url.pathname));

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

  $effect(() => {
    if (authStore.isAuthenticated === true) {
      void Promise.all([familyStore.load(), categoryStore.load()]);
    } else {
      familyStore.reset();
      categoryStore.reset();
    }
  });
</script>

{#if authStore.isLoading && !isPublicRoute}
  <!-- Centered OLED loading spinner for protected routes during auth initialization -->
  <div class="bg-background flex min-h-screen items-center justify-center">
    <div
      class="border-border/40 size-8 animate-spin rounded-full border-2 border-t-emerald-500"
    ></div>
  </div>
{:else if authStore.isAuthenticated}
  <Sidebar.Provider
    bind:open={sidebarOpen}
    onOpenChange={handleSidebarOpenChange}
    style="--sidebar-width-icon: 4rem;"
  >
    <AppSidebar />
    <Sidebar.Inset class="flex h-svh max-h-svh flex-col overflow-hidden">
      <header
        class="border-border/40 bg-background/80 sticky top-0 z-10 flex h-12 shrink-0 items-center gap-2 border-b px-4 backdrop-blur-xs"
      >
        <Sidebar.Trigger class="text-muted-foreground hover:text-foreground -ml-1" />
        <Separator orientation="vertical" class="mr-2 h-4" />
      </header>
      <main class="flex min-h-0 flex-1 flex-col overflow-y-auto p-4 md:p-6">
        {@render children()}
      </main>
    </Sidebar.Inset>
  </Sidebar.Provider>
{:else if isPublicRoute}
  <div class="bg-background text-foreground flex min-h-screen flex-col">
    <TopNav />
    <main class="flex flex-1 flex-col">
      {@render children()}
    </main>
  </div>
{:else}
  <!-- In-Place Auth Wall on protected route: renders standalone login card without route bounce or DOM leakage -->
  <div class="bg-background flex min-h-screen items-center justify-center p-4">
    <AuthModal embedded={true} />
  </div>
{/if}
