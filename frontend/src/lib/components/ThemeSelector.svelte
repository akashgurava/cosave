<script lang="ts">
  import { themeStore, type ThemeMode } from "$lib/theme";

  interface ThemeOption {
    id: ThemeMode;
    title: string;
    description: string;
  }

  const themeOptions: ThemeOption[] = [
    {
      id: "dark",
      title: "Dark (OLED)",
      description: "Pitch black (#000000) with low-opacity borders and crisp white text",
    },
    {
      id: "light",
      title: "Pure Light",
      description: "Clean white (#ffffff) with subtle dark borders and deep black text",
    },
    {
      id: "system",
      title: "System Preference",
      description: "Automatically synchronizes with your device operating system theme",
    },
  ];
</script>

<div class="w-full rounded-2xl border border-(--border-subtle) bg-(--bg-surface) p-6">
  <div class="mb-5">
    <h2 class="text-base font-semibold text-(--text-primary)">Appearance</h2>
    <p class="mt-1 text-xs text-(--text-secondary)">
      Select your preferred color theme or match your system settings.
    </p>
  </div>

  <div class="grid grid-cols-1 gap-3 sm:grid-cols-3" role="radiogroup" aria-label="Theme selection">
    {#each themeOptions as option (option.id)}
      {@const isSelected = themeStore.mode === option.id}
      <button
        type="button"
        role="radio"
        aria-checked={isSelected}
        class="group relative flex cursor-pointer flex-col items-start rounded-xl border p-4 text-left transition-all {isSelected
          ? 'border-emerald-500 bg-emerald-500/10 shadow-sm'
          : 'border-(--border-subtle) bg-transparent hover:border-(--border-strong) hover:bg-(--bg-hover)'}"
        onclick={() => themeStore.setTheme(option.id)}
      >
        <!-- Preview swatch icon -->
        <div class="mb-3 flex w-full items-center justify-between">
          <div
            class="flex size-7 items-center justify-center rounded-lg border {option.id === 'dark'
              ? 'border-white/20 bg-black text-white'
              : option.id === 'light'
                ? 'border-black/15 bg-white text-black'
                : 'border-(--border-subtle) bg-(--bg-hover) text-(--text-primary)'}"
          >
            {#if option.id === "dark"}
              <svg
                class="size-3.5"
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
                class="size-3.5"
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
                class="size-3.5"
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
          </div>

          {#if isSelected}
            <span
              class="flex size-4 items-center justify-center rounded-full bg-emerald-500 text-white"
            >
              <svg
                class="size-2.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
                aria-hidden="true"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </span>
          {/if}
        </div>

        <span class="text-xs font-semibold text-(--text-primary)">
          {option.title}
        </span>
        <span class="mt-1 text-[11px] leading-relaxed text-(--text-secondary)">
          {option.description}
        </span>
      </button>
    {/each}
  </div>
</div>
