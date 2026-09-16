<script lang="ts">
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props extends HTMLAttributes<HTMLHeadingElement> {
    children?: Snippet;
  }

  let { class: className = "", children, ...restProps }: Props = $props();
</script>

<h3
  class={cn("text-base font-semibold tracking-tight text-(--text-primary)", className)}
  {...restProps}
>
  {@render children?.()}
</h3>
