<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import BackendStatusDot from "./BackendStatusDot.svelte";

  interface NavItem {
    label: string;
    href: string;
  }

  const navItems: NavItem[] = [
    { label: "Dashboard", href: "/" },
    { label: "Budgets", href: "/#budgets" },
    { label: "Vaults", href: "/#vaults" },
    { label: "Settings", href: "/settings" },
  ];

  let currentPath = $derived(page.url.pathname);
</script>

<header
  class="sticky top-0 z-40 w-full border-b border-(--border-subtle) bg-(--bg-glass) px-6 py-3.5 backdrop-blur-md transition-colors"
>
  <div class="mx-auto flex max-w-6xl items-center justify-between">
    <!-- Left: Brand & Beacon Dot (no divider, small gap) -->
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

    <!-- Right: Text Navigation with Underline Indicator -->
    <nav class="flex items-center gap-6" aria-label="Main Navigation">
      {#each navItems as item (item.label)}
        {@const isActive =
          (item.href === "/" && currentPath === "/") ||
          (item.href === "/settings" && currentPath === "/settings")}
        <a
          href={resolve(item.href as "/" | "/settings")}
          class="relative pb-1 text-xs font-medium transition-colors {isActive
            ? 'text-(--text-primary)'
            : 'text-(--text-secondary) hover:text-(--text-primary)'}"
          aria-current={isActive ? "page" : undefined}
        >
          {item.label}
          {#if isActive}
            <span
              class="absolute -bottom-1 left-0 h-0.5 w-full rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.7)]"
            ></span>
          {/if}
        </a>
      {/each}
    </nav>
  </div>
</header>
