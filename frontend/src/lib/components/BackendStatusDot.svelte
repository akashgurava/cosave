<script lang="ts">
  import { healthStore } from "$lib/health";

  let showPopover = $state(false);

  function togglePopover(): void {
    showPopover = !showPopover;
  }

  function closePopover(): void {
    showPopover = false;
  }
</script>

<div class="relative inline-flex items-center">
  <!-- Interactive Beacon Button with touch & hover support -->
  <button
    type="button"
    id="backend-status-dot"
    aria-label={healthStore.isOnline ? "Backend status: online" : "Backend status: offline"}
    aria-expanded={showPopover}
    class="group relative flex size-7 cursor-pointer items-center justify-center rounded-full transition-transform hover:scale-110 focus:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/50"
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

  <!-- Explanation Tooltip/Popover on hover or touch -->
  {#if showPopover}
    <div
      role="tooltip"
      id="backend-status-popover"
      class="absolute top-full left-0 z-50 mt-2 w-64 rounded-xl border border-(--border-subtle) bg-(--bg-glass) p-3 shadow-2xl backdrop-blur-xl sm:left-1/2 sm:-translate-x-1/2"
      onmouseenter={() => (showPopover = true)}
      onmouseleave={closePopover}
    >
      <div class="flex items-start gap-2.5">
        <div
          class="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full {healthStore.isOnline
            ? 'bg-emerald-500/20 text-emerald-400'
            : 'bg-rose-500/20 text-rose-400'}"
        >
          <span
            class="size-1.5 rounded-full {healthStore.isOnline ? 'bg-emerald-400' : 'bg-rose-400'}"
          ></span>
        </div>
        <div class="flex-1 text-left">
          <div class="flex items-center justify-between">
            <p class="text-xs font-semibold text-(--text-primary)">
              {healthStore.isOnline ? "Backend Online" : "Backend Offline"}
            </p>
            <span
              class="rounded px-1.5 py-0.5 text-[10px] font-medium {healthStore.isOnline
                ? 'bg-emerald-500/10 text-emerald-400'
                : 'bg-rose-500/10 text-rose-400'}"
            >
              {healthStore.isOnline ? "Healthy (200)" : "Unreachable"}
            </span>
          </div>
          <p class="mt-1 text-[11px] leading-relaxed text-(--text-secondary)">
            {healthStore.isOnline
              ? "Rust Axum API service is operational and serving endpoints."
              : "Unable to reach http://localhost:3000/api/v1/health. Check backend server."}
          </p>
          {#if healthStore.lastChecked}
            <p
              class="mt-2 border-t border-(--border-subtle) pt-1.5 text-[10px] text-(--text-muted)"
            >
              Last ping: {healthStore.lastChecked.toLocaleTimeString()}
            </p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
