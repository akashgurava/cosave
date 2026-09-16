<script lang="ts" module>
  import type { HTMLButtonAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  export type ButtonVariant =
    "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";

  export type ButtonSize = "default" | "sm" | "lg" | "icon";

  export interface ButtonProps extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: ButtonSize;
    children?: Snippet;
  }
</script>

<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    variant = "default",
    size = "default",
    class: className = "",
    type = "button",
    children,
    disabled = false,
    ...restProps
  }: ButtonProps = $props();

  const variantStyles: Record<ButtonVariant, string> = {
    default:
      "bg-(--btn-primary-bg) text-(--btn-primary-text) font-bold shadow-md hover:bg-(--btn-primary-hover) active:opacity-90",
    destructive: "bg-rose-600 text-white shadow-sm hover:bg-rose-500 active:bg-rose-700",
    outline:
      "border border-(--border-subtle) bg-transparent text-(--text-primary) hover:border-(--border-strong) hover:bg-(--bg-hover)",
    secondary: "bg-(--bg-hover) text-(--text-primary) hover:bg-(--border-subtle)",
    ghost: "text-(--text-secondary) hover:bg-(--bg-hover) hover:text-(--text-primary)",
    link: "text-emerald-500 underline-offset-4 hover:underline",
  };

  const sizeStyles: Record<ButtonSize, string> = {
    default: "h-9 px-4 py-2 text-xs rounded-xl",
    sm: "h-8 px-3 text-[11px] rounded-lg",
    lg: "h-11 px-6 text-sm rounded-xl",
    icon: "size-8 p-0 rounded-xl",
  };
</script>

<button
  {type}
  {disabled}
  class={cn(
    "inline-flex cursor-pointer items-center justify-center font-semibold transition-all focus-visible:ring-2 focus-visible:ring-(--text-primary) focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50",
    variantStyles[variant],
    sizeStyles[size],
    className,
  )}
  {...restProps}
>
  {@render children?.()}
</button>
