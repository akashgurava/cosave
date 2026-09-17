<script lang="ts">
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import { authStore } from "$lib/auth";
  import BackendStatusDot from "./BackendStatusDot.svelte";

  let isMobileOpen = $state(false);

  function toggleMobile(): void {
    isMobileOpen = !isMobileOpen;
  }

  function closeMobile(): void {
    isMobileOpen = false;
  }
</script>

<!-- Mobile Top Bar (< md) -->
<div
  class="flex h-14 w-full items-center justify-between border-b border-(--border-subtle) bg-(--bg-glass) px-4 backdrop-blur-md md:hidden"
>
  <a href={resolve("/")} class="flex items-center gap-2.5">
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

  <div class="flex items-center gap-2">
    <BackendStatusDot />
    <button
      type="button"
      onclick={toggleMobile}
      aria-label="Toggle navigation menu"
      class="flex size-9 cursor-pointer items-center justify-center rounded-xl text-(--text-secondary) transition-colors hover:bg-(--bg-hover) hover:text-(--text-primary)"
    >
      {#if isMobileOpen}
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      {:else}
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="3" y1="12" x2="21" y2="12" />
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
      {/if}
    </button>
  </div>
</div>

<!-- Mobile Backdrop Overlay -->
{#if isMobileOpen}
  <button
    type="button"
    tabindex="-1"
    aria-label="Close sidebar overlay"
    class="fixed inset-0 z-30 bg-black/60 backdrop-blur-xs md:hidden"
    onclick={closeMobile}
  ></button>
{/if}

<!-- Sidebar Navigation -->
<aside
  class="fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-(--border-subtle) bg-(--bg-surface) p-4 transition-transform duration-200 ease-in-out md:static md:translate-x-0 {isMobileOpen
    ? 'translate-x-0 shadow-2xl'
    : '-translate-x-full'}"
>
  <!-- Top: Brand Header -->
  <div class="flex items-center justify-between p-2">
    <a href={resolve("/")} class="flex items-center gap-2.5" onclick={closeMobile}>
      <div
        class="flex size-8 items-center justify-center rounded-xl bg-(image:--brand-gradient) text-white shadow-md"
      >
        <svg
          class="size-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
        </svg>
      </div>
      <div>
        <span class="text-sm font-bold tracking-tight text-(--text-primary)">CoSave</span>
      </div>
    </a>
    <BackendStatusDot />
  </div>

  <!-- Top Navigation Items -->
  <nav class="mt-6 flex-1 space-y-1">
    <a
      href={resolve("/")}
      onclick={closeMobile}
      class="flex items-center gap-3 rounded-xl px-3 py-2.5 text-xs font-medium transition-all {page
        .url.pathname === '/'
        ? 'border-l-2 border-emerald-500 bg-emerald-500/10 font-semibold text-emerald-400'
        : 'text-(--text-secondary) hover:bg-(--bg-hover) hover:text-(--text-primary)'}"
    >
      <svg
        class="size-4.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
        <polyline points="9 22 9 12 15 12 15 22" />
      </svg>
      <span>Home</span>
    </a>
  </nav>

  <!-- Bottom: Settings & Log Out -->
  <div class="mt-auto space-y-1 border-t border-(--border-subtle) pt-3">
    {#if authStore.currentUser}
      {@const user = authStore.currentUser}
      <!-- User Profile Summary Pill -->
      <div class="mb-2 flex items-center gap-2.5 rounded-xl bg-(--bg-hover) px-3 py-2">
        <div
          class="flex size-7 items-center justify-center rounded-lg bg-emerald-500/20 text-xs font-bold text-emerald-400"
        >
          {user.name.slice(0, 2).toUpperCase()}
        </div>
        <div class="min-w-0 flex-1">
          <p class="truncate text-xs font-semibold text-(--text-primary)">{user.name}</p>
          <span class="text-[10px] tracking-wide text-(--text-muted) uppercase">
            {user.role}
          </span>
        </div>
      </div>
    {/if}

    <a
      href={resolve("/settings")}
      onclick={closeMobile}
      class="flex items-center gap-3 rounded-xl px-3 py-2.5 text-xs font-medium transition-all {page.url.pathname.startsWith(
        '/settings',
      )
        ? 'border-l-2 border-emerald-500 bg-emerald-500/10 font-semibold text-emerald-400'
        : 'text-(--text-secondary) hover:bg-(--bg-hover) hover:text-(--text-primary)'}"
    >
      <svg
        class="size-4.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
        />
        <circle cx="12" cy="12" r="3" />
      </svg>
      <span>Settings</span>
    </a>

    <button
      type="button"
      onclick={() => {
        closeMobile();
        authStore.logout();
      }}
      class="flex w-full cursor-pointer items-center gap-3 rounded-xl px-3 py-2.5 text-xs font-medium text-(--text-secondary) transition-colors hover:bg-rose-500/10 hover:text-rose-400"
    >
      <svg
        class="size-4.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
        <polyline points="16 17 21 12 16 7" />
        <line x1="21" y1="12" x2="9" y2="12" />
      </svg>
      <span>Log Out</span>
    </button>
  </div>
</aside>
