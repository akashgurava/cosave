<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props extends HTMLAttributes<HTMLDivElement> {
    children?: Snippet;
  }

  let { class: className = "", children, ...restProps }: Props = $props();
</script>

<div
  class={cn(
    "rounded-2xl border border-(--border-subtle) bg-(--bg-surface) shadow-sm transition-all",
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</div>
