<script lang="ts">
  interface Props {
    isOnline: boolean;
    onlineLabel?: string;
    offlineLabel?: string;
  }

  let {
    isOnline,
    onlineLabel = "Backend Online",
    offlineLabel = "Backend Offline",
  }: Props = $props();
</script>

{#if isOnline}
  <span
    id="status-badge-online"
    class="inline-flex items-center gap-2.5 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-sm font-semibold text-emerald-400 shadow-lg shadow-emerald-500/5"
  >
    <span class="size-2.5 animate-pulse rounded-full bg-emerald-400" aria-hidden="true"></span>
    {onlineLabel}
  </span>
{:else}
  <span
    id="status-badge-offline"
    class="inline-flex items-center gap-2.5 rounded-full border border-rose-500/30 bg-rose-500/10 px-4 py-2 text-sm font-semibold text-rose-400 shadow-lg shadow-rose-500/5"
  >
    <span class="size-2.5 rounded-full bg-rose-400" aria-hidden="true"></span>
    {offlineLabel}
  </span>
{/if}
