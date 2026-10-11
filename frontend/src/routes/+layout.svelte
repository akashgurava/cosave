<script lang="ts">
  import "../app.css";
  import { TopNav, AppSidebar } from "$components";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator";
  import { authStore, AuthModal } from "$lib/features/auth";
  import { familyStore, CreateFamilyModal } from "$lib/features/family";
  import { healthStore } from "$lib/health.svelte";
  import { themeStore } from "$lib/theme.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // 1. Global Constants
  const PUBLIC_ROUTES = [resolve("/")];
  const SIDEBAR_STORAGE_KEY = "cosave_sidebar_open";

  // 2. Route Derivations
  const isPublicRoute = $derived(PUBLIC_ROUTES.includes(page.url.pathname));

  // 3. Sidebar State & Handlers
  function getInitialSidebarOpen(): boolean {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem(SIDEBAR_STORAGE_KEY);
      if (saved !== null) {
        return saved === "true";
      }
    }
    return true;
  }

  let sidebarOpen = $state<boolean>(getInitialSidebarOpen());

  function handleSidebarOpenChange(open: boolean) {
    sidebarOpen = open;
    if (typeof window !== "undefined") {
      localStorage.setItem(SIDEBAR_STORAGE_KEY, String(open));
    }
  }

  // 4. Session & Tenant Bootstrap Derivation
  const isBootstrapping = $derived(
    authStore.isLoading === true ||
      (authStore.isAuthenticated === true && familyStore.isSuccess === false),
  );

  // 5. Lifecycle Effects
  $effect(() => {
    themeStore.applyTheme();
  });

  $effect(() => {
    return healthStore.startPolling(30000);
  });

  $effect(() => {
    if (authStore.isAuthenticated === true) {
      void familyStore.load();
    } else {
      familyStore.reset();
    }
  });
</script>

{#if isBootstrapping}
  <!-- Centered loading spinner during auth & tenant initialization -->
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
        <svelte:boundary>
          {@render children()}
          {#snippet failed(error, reset)}
            <div class="rounded-xl border border-destructive/20 bg-destructive/5 p-6 text-center">
              <p class="text-base font-semibold text-destructive">Failed to render page</p>
              <p class="mt-1 text-xs text-muted-foreground">
                {error instanceof Error ? error.message : "An unexpected rendering error occurred"}
              </p>
              <button
                type="button"
                class="mt-4 inline-flex items-center rounded-md border px-3 py-1.5 text-xs font-medium hover:bg-muted"
                onclick={reset}
              >
                Retry
              </button>
            </div>
          {/snippet}
        </svelte:boundary>
      </main>
    </Sidebar.Inset>
  </Sidebar.Provider>

  <CreateFamilyModal open={familyStore.needsOnboarding} />
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

<Toaster position="bottom-right" richColors />
