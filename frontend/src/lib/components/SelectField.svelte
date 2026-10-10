<script module lang="ts">
  export interface SelectFieldItem<V = string | number> {
    value: V;
    label: string;
    sublabel?: string;
    color?: string;
    disabled?: boolean;
  }
</script>

<script lang="ts" generics="T = string | number">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { CheckIcon, ChevronDownIcon } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  interface Props {
    id: string;
    label: string;
    required?: boolean;
    items: readonly SelectFieldItem<T>[];
    value?: T | undefined;
    placeholder?: string;
    allowEmpty?: boolean;
    emptyLabel?: string;
    disabled?: boolean;
    class?: string;
    triggerClass?: string;
    contentClass?: string;
    error?: string | null;
    onchange?: (value: T | undefined) => void;
  }

  let {
    id,
    label,
    required = false,
    items,
    value = $bindable(undefined),
    placeholder = "Select...",
    allowEmpty = false,
    emptyLabel = "(None)",
    disabled = false,
    class: className = "",
    triggerClass = "",
    contentClass = "",
    error = null,
    onchange,
  }: Props = $props();

  const selectedItem = $derived(items.find((item) => item.value === value));

  function handleSelect(newVal: T | undefined) {
    value = newVal;
    onchange?.(newVal);
  }
</script>

<div class={cn("flex flex-col gap-1.5", className)}>
  <label for={id} class="text-muted-foreground text-xs font-semibold">
    {label}
    {#if required}
      <span class="text-destructive">*</span>
    {/if}
  </label>

  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      {id}
      {disabled}
      class={cn(
        "border-border/40 bg-background flex h-8 w-full items-center justify-between rounded-md border px-2.5 text-xs outline-none transition-colors hover:bg-muted/30 focus-visible:ring-1 focus-visible:ring-foreground/20 disabled:cursor-not-allowed disabled:opacity-50",
        triggerClass,
      )}
    >
      <span class="flex items-center gap-2 truncate">
        {#if selectedItem?.color}
          <span
            class="size-2 shrink-0 rounded-full"
            style="background-color: {selectedItem.color};"
          ></span>
        {/if}
        <span class="truncate {selectedItem !== undefined ? 'text-foreground' : 'text-muted-foreground'}">
          {selectedItem?.label ?? placeholder}
        </span>
      </span>
      <ChevronDownIcon class="text-muted-foreground size-3.5 shrink-0 opacity-60" />
    </DropdownMenu.Trigger>

    <DropdownMenu.Content align="start" class={cn("z-50 max-h-56 overflow-y-auto p-1", contentClass)}>
      {#if allowEmpty}
        <DropdownMenu.Item
          class="flex cursor-pointer items-center justify-between px-2 py-1.5 text-xs"
          onclick={() => handleSelect(undefined)}
        >
          <span class="text-muted-foreground">{emptyLabel}</span>
          {#if value === undefined}
            <CheckIcon class="text-foreground size-3.5 shrink-0" />
          {/if}
        </DropdownMenu.Item>
      {/if}

      {#each items as item (item.value)}
        <DropdownMenu.Item
          disabled={item.disabled}
          class="flex cursor-pointer items-center justify-between px-2 py-1.5 text-xs"
          onclick={() => handleSelect(item.value)}
        >
          <div class="flex items-center gap-2 truncate">
            {#if item.color}
              <span
                class="size-2 shrink-0 rounded-full"
                style="background-color: {item.color};"
              ></span>
            {/if}
            <div class="truncate">
              <div>{item.label}</div>
              {#if item.sublabel}
                <div class="text-muted-foreground font-mono text-[10px]">{item.sublabel}</div>
              {/if}
            </div>
          </div>
          {#if value === item.value}
            <CheckIcon class="text-foreground size-3.5 shrink-0" />
          {/if}
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Root>

  {#if error !== null && error !== undefined && error.length > 0}
    <p class="text-destructive text-xs font-medium">{error}</p>
  {/if}
</div>
