<script lang="ts">
  import { resolve } from "$app/paths";
  import { authStore } from "$lib/auth";
  import BackendStatusDot from "./BackendStatusDot.svelte";
  import { AuthModal, UserMenu } from "$lib/features/auth";
  import { Button } from "$lib/components/ui/button";

  let showAuthModal = $state(false);
</script>

<header
  class="sticky top-0 z-40 w-full border-b border-(--border-subtle) bg-(--bg-glass) px-6 py-3.5 backdrop-blur-md transition-colors"
>
  <div class="mx-auto flex max-w-6xl items-center justify-between">
    <!-- Left: Brand & Server Status Beacon -->
    <div class="flex items-center gap-2.5">
      <a href={resolve("/")} class="flex items-center gap-2">
        <div
          class="flex size-7 items-center justify-center rounded-lg bg-(image:--brand-gradient) text-white shadow-sm"
        >
          <svg
            class="size-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            aria-hidden="true"
          >
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
          </svg>
        </div>
        <span class="text-sm font-bold tracking-tight text-(--text-primary)">CoSave</span>
      </a>
      <BackendStatusDot />
    </div>

    <!-- Right: Authentication & User Profile Action -->
    <div class="flex items-center gap-3">
      {#if authStore.isLoading}
        <div class="size-6 animate-pulse rounded-full bg-(--border-subtle)"></div>
      {:else if authStore.isAuthenticated}
        <UserMenu />
      {:else}
        <Button size="sm" onclick={() => (showAuthModal = true)} class="font-bold shadow-md">
          Sign In
        </Button>
      {/if}
    </div>
  </div>
</header>

<AuthModal isOpen={showAuthModal} onClose={() => (showAuthModal = false)} />
