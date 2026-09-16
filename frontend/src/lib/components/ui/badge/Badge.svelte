<script lang="ts" module>
  import type { HTMLAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  export type BadgeVariant =
    "default" | "secondary" | "destructive" | "outline" | "success" | "warning";

  export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
    variant?: BadgeVariant;
    children?: Snippet;
  }
</script>

<script lang="ts">
  import { cn } from "$lib/utils";

  let { variant = "default", class: className = "", children, ...restProps }: BadgeProps = $props();

  const variantStyles: Record<BadgeVariant, string> = {
    default: "bg-emerald-500/15 text-emerald-400 border border-emerald-500/30",
    secondary: "bg-(--bg-hover) text-(--text-secondary) border border-(--border-subtle)",
    destructive: "bg-rose-500/15 text-rose-400 border border-rose-500/30",
    outline: "border border-(--border-subtle) text-(--text-primary)",
    success: "bg-emerald-500/15 text-emerald-400 border border-emerald-500/30",
    warning: "bg-amber-500/15 text-amber-400 border border-amber-500/30",
  };
</script>

<span
  class={cn(
    "inline-flex items-center rounded-md px-2 py-0.5 text-[10px] font-semibold tracking-wider uppercase transition-colors",
    variantStyles[variant],
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</span>
