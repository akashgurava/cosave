<script lang="ts">
  import { healthStore } from "$lib/health";
  import type { SvelteDate } from "svelte/reactivity";

  let showPopover = $state(false);
  let now = $state(Date.now());

  $effect(() => {
    const interval = setInterval(() => {
      now = Date.now();
    }, 1000);
    return () => clearInterval(interval);
  });

  function formatElapsedTime(date: SvelteDate | Date | null): string {
    if (!date) return "Checking...";
    const diffSec = Math.max(0, Math.floor((now - date.getTime()) / 1000));
    if (diffSec < 5) return "just now";
    if (diffSec < 60) return `${diffSec}s ago`;
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHour = Math.floor(diffMin / 60);
    return `${diffHour}h ago`;
  }

  function togglePopover(): void {
    showPopover = !showPopover;
  }
</script>

<div class="relative inline-flex items-center">
  <!-- Interactive Beacon Button with touch & hover support -->
  <button
    type="button"
    id="backend-status-dot"
    aria-label={healthStore.isOnline ? "Service online" : "Service unavailable"}
    aria-expanded={showPopover}
    class="group relative flex size-7 cursor-pointer items-center justify-center rounded-full transition-transform hover:scale-110 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500/50"
    onclick={togglePopover}
    onmouseenter={() => (showPopover = true)}
    onmouseleave={() => (showPopover = false)}
  >
    {#if healthStore.isOnline}
      <!-- Ping radar ring -->
      <span
        class="absolute inline-flex size-3.5 animate-ping rounded-full bg-emerald-400 opacity-60"
        aria-hidden="true"
      ></span>
      <!-- Glowing emerald dot -->
      <span
        class="relative inline-flex size-3 rounded-full bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,0.8)] transition-all group-hover:scale-125"
        aria-hidden="true"
      ></span>
    {:else}
      <!-- Offline pulsing ring -->
      <span
        class="absolute inline-flex size-3.5 animate-pulse rounded-full bg-rose-500 opacity-50"
        aria-hidden="true"
      ></span>
      <!-- Glowing rose dot -->
      <span
        class="relative inline-flex size-3 rounded-full bg-rose-500 shadow-[0_0_12px_rgba(244,63,94,0.8)] transition-all group-hover:scale-125"
        aria-hidden="true"
      ></span>
    {/if}
  </button>

  <!-- Clean, non-technical tooltip with live elapsed time -->
  {#if showPopover}
    <div
      role="tooltip"
      id="backend-status-popover"
      class="pointer-events-none absolute top-full left-0 z-50 mt-2 w-48 rounded-xl border border-(--border-subtle) bg-(--bg-glass) p-2.5 shadow-xl backdrop-blur-xl sm:left-1/2 sm:-translate-x-1/2"
    >
      <div class="flex items-center gap-2.5">
        <span
          class="size-2 shrink-0 rounded-full {healthStore.isOnline
            ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]'
            : 'bg-rose-400 shadow-[0_0_8px_rgba(244,63,94,0.8)]'}"
        ></span>
        <div class="min-w-0 flex-1 text-left">
          <p class="truncate text-xs font-semibold text-(--text-primary)">
            {healthStore.isOnline ? "Service Online" : "Service Unavailable"}
          </p>
          {#if healthStore.lastChecked}
            <p class="text-[11px] text-(--text-secondary)">
              Updated {formatElapsedTime(healthStore.lastChecked)}
            </p>
          {:else}
            <p class="text-[11px] text-(--text-secondary)">Checking connection...</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
