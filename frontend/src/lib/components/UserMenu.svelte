<script lang="ts">
  import { resolve } from "$app/paths";
  import { authStore } from "$lib/auth";

  let showMenu = $state(false);

  function toggleMenu(): void {
    showMenu = !showMenu;
  }

  function closeMenu(): void {
    showMenu = false;
  }

  async function handleLogout(): Promise<void> {
    closeMenu();
    await authStore.logout();
  }

  let user = $derived(authStore.currentUser);
  let initials = $derived(user && user.name ? user.name.slice(0, 2).toUpperCase() : "U");
</script>

{#if user}
  <div class="relative inline-flex items-center">
    <button
      type="button"
      id="user-menu-button"
      aria-expanded={showMenu}
      aria-haspopup="true"
      onclick={toggleMenu}
      class="flex cursor-pointer items-center gap-2 rounded-xl border border-(--border-subtle) bg-(--bg-surface) py-1.5 pr-2.5 pl-2 transition-colors hover:border-(--border-strong) hover:bg-(--bg-hover)"
    >
      <div
        class="flex size-6 items-center justify-center rounded-lg bg-emerald-500/20 text-[11px] font-bold text-emerald-400"
      >
        {initials}
      </div>
      <span class="max-w-25 truncate text-xs font-medium text-(--text-primary)">
        {user.name}
      </span>
      <span
        class="rounded px-1.5 py-0.5 text-[9px] font-semibold tracking-wider uppercase {user.role ===
        'admin'
          ? 'border border-amber-500/30 bg-amber-500/15 text-amber-400'
          : 'bg-(--border-subtle) text-(--text-muted)'}"
      >
        {user.role}
      </span>
      <svg
        class="size-3 text-(--text-muted) transition-transform {showMenu ? 'rotate-180' : ''}"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>

    {#if showMenu}
      <!-- Backdrop to close on outer click -->
      <div class="fixed inset-0 z-40" onclick={closeMenu} aria-hidden="true"></div>

      <!-- Dropdown -->
      <div
        class="absolute top-full right-0 z-50 mt-2 w-52 rounded-xl border border-(--border-subtle) bg-(--bg-surface) p-2 shadow-2xl backdrop-blur-xl"
        role="menu"
        aria-orientation="vertical"
        aria-labelledby="user-menu-button"
      >
        <div class="border-b border-(--border-subtle) px-2.5 py-2">
          <p class="text-xs font-semibold text-(--text-primary)">{user.name}</p>
          {#if user.email}
            <p class="truncate text-[11px] text-(--text-secondary)">{user.email}</p>
          {/if}
        </div>

        <div class="py-1">
          <a
            href={resolve("/settings")}
            onclick={closeMenu}
            class="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium text-(--text-secondary) transition-colors hover:bg-(--bg-hover) hover:text-(--text-primary)"
            role="menuitem"
          >
            <svg
              class="size-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="3" />
              <path
                d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
              />
            </svg>
            Settings
          </a>

          <button
            type="button"
            onclick={handleLogout}
            class="flex w-full cursor-pointer items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium text-rose-400 transition-colors hover:bg-rose-500/10"
            role="menuitem"
          >
            <svg
              class="size-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line x1="21" y1="12" x2="9" y2="12" />
            </svg>
            Sign Out
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
