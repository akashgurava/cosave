<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import { Calendar } from "$lib/components/ui/calendar";
  import { CalendarIcon } from "@lucide/svelte";
  import { type DateValue, parseDate } from "@internationalized/date";
  import { cn } from "$lib/utils";

  interface Props {
    id?: string;
    label?: string;
    required?: boolean;
    value?: string;
    disabled?: boolean;
    class?: string;
    triggerClass?: string;
    showIcon?: boolean;
    onchange?: (val: string) => void;
  }

  let {
    id = "date-field",
    label = undefined,
    required = false,
    value = $bindable(""),
    disabled = false,
    class: className = "",
    triggerClass = "",
    showIcon = true,
    onchange,
  }: Props = $props();

  let isOpen = $state(false);

  const calendarDateValue = $derived.by(() => {
    if (!value) return undefined;
    try {
      return parseDate(value);
    } catch {
      return undefined;
    }
  });

  function handleSelect(val: DateValue | undefined) {
    if (val) {
      const str = val.toString();
      value = str;
      onchange?.(str);
      isOpen = false;
    }
  }
</script>

{#snippet trigger()}
  <Popover.Root bind:open={isOpen}>
    <Popover.Trigger
      {id}
      {disabled}
      class={cn(
        label !== undefined && label.length > 0
          ? "border-border/40 bg-background flex h-8 w-full items-center justify-between rounded-md border px-2.5 font-mono text-xs outline-none transition-colors hover:bg-muted/30 focus-visible:ring-1 focus-visible:ring-foreground/20 disabled:cursor-not-allowed disabled:opacity-50"
          : "text-muted-foreground hover:bg-muted/60 hover:text-foreground w-full rounded px-1 py-0.5 text-left font-mono text-[11px] transition-colors",
        triggerClass,
      )}
    >
      <span>{value || "Select date"}</span>
      {#if showIcon}
        <CalendarIcon class="text-muted-foreground size-3.5 shrink-0 opacity-60" />
      {/if}
    </Popover.Trigger>
    <Popover.Content
      align="start"
      side="bottom"
      sideOffset={4}
      class="bg-popover border-border/40 z-50 w-auto border p-0 shadow-lg"
    >
      <Calendar
        type="single"
        preventDeselect
        value={calendarDateValue}
        onValueChange={handleSelect}
      />
    </Popover.Content>
  </Popover.Root>
{/snippet}

{#if label !== undefined && label.length > 0}
  <div class={cn("flex flex-col gap-1.5", className)}>
    <label for={id} class="text-muted-foreground text-xs font-semibold">
      {label}
      {#if required}
        <span class="text-destructive">*</span>
      {/if}
    </label>
    {@render trigger()}
  </div>
{:else}
  {@render trigger()}
{/if}
