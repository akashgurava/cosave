<script lang="ts">
  import { themeStore, type ThemeMode } from "$lib/theme.svelte";

  interface ThemeOption {
    id: ThemeMode;
    title: string;
  }

  const themeOptions: ThemeOption[] = [
    {
      id: "dark",
      title: "Dark (OLED)",
    },
    {
      id: "light",
      title: "Light",
    },
    {
      id: "system",
      title: "System Preference",
    },
  ];
</script>

<div
  class="flex w-full flex-col justify-between gap-4 rounded-xl border border-(--border-subtle) bg-(--bg-surface) p-4 sm:flex-row sm:items-center sm:p-5"
>
  <h2 class="text-sm font-semibold text-(--text-primary)">Theme Preference</h2>

  <div class="flex flex-wrap items-center gap-2" role="radiogroup" aria-label="Theme selection">
    {#each themeOptions as option (option.id)}
      {@const isSelected = themeStore.mode === option.id}
      <button
        type="button"
        role="radio"
        aria-checked={isSelected}
        class="flex h-8 w-full cursor-pointer items-center justify-center gap-2 rounded-lg border px-3 text-xs font-medium transition-all sm:w-36 {isSelected ===
        true
          ? 'border-foreground bg-foreground text-background shadow-xs'
          : 'border-(--border-subtle) bg-transparent text-(--text-secondary) hover:border-(--border-strong) hover:bg-(--bg-hover)/50 hover:text-(--text-primary)'}"
        onclick={() => themeStore.setTheme(option.id)}
      >
        <!-- Preview swatch icon -->
        <span
          class="flex size-4 items-center justify-center rounded border {option.id === 'dark'
            ? 'border-white/20 bg-black text-white'
            : option.id === 'light'
              ? 'border-black/15 bg-white text-black'
              : isSelected === true
                ? 'border-current/30 bg-current/10 text-current'
                : 'border-(--border-subtle) bg-(--bg-hover) text-(--text-primary)'}"
        >
          {#if option.id === "dark"}
            <svg
              class="size-2.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
            </svg>
          {:else if option.id === "light"}
            <svg
              class="size-2.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <circle cx="12" cy="12" r="4" />
              <path d="M12 2v2" />
              <path d="M12 20v2" />
              <path d="m4.93 4.93 1.41 1.41" />
              <path d="m17.66 17.66 1.41 1.41" />
              <path d="M2 12h2" />
              <path d="M20 12h2" />
              <path d="m6.34 17.66-1.41 1.41" />
              <path d="m19.07 4.93-1.41 1.41" />
            </svg>
          {:else}
            <svg
              class="size-2.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <rect width="20" height="14" x="2" y="3" rx="2" />
              <line x1="8" x2="16" y1="21" y2="21" />
              <line x1="12" x2="12" y1="17" y2="21" />
            </svg>
          {/if}
        </span>

        <span class="whitespace-nowrap">{option.title}</span>
      </button>
    {/each}
  </div>
</div>
