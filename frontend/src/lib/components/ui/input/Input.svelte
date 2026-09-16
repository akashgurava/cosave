<script lang="ts" module>
  import type { HTMLInputAttributes } from "svelte/elements";

  export interface InputProps extends HTMLInputAttributes {
    value?: string | number | string[];
  }
</script>

<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    class: className = "",
    value = $bindable(),
    type = "text",
    ...restProps
  }: InputProps = $props();
</script>

<input
  {type}
  bind:value
  class={cn(
    "flex h-9 w-full rounded-xl border border-(--border-subtle) bg-(--bg-hover) px-3.5 py-2 text-xs text-(--text-primary) shadow-sm transition-colors file:border-0 file:bg-transparent file:text-xs file:font-medium placeholder:text-(--text-muted) focus-visible:border-emerald-500 focus-visible:ring-1 focus-visible:ring-emerald-500 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  {...restProps}
/>
