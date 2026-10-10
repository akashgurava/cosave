<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { cn } from "$lib/utils";

  interface Props {
    id?: string;
    label?: string;
    required?: boolean;
    isAmount?: boolean;
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    error?: string | null;
    autofocus?: boolean;
    class?: string;
    inputClass?: string;
    onkeydown?: (e: KeyboardEvent) => void;
    oninput?: (e: Event) => void;
    onblur?: (e: FocusEvent) => void;
  }

  let {
    id = undefined,
    label = undefined,
    required = false,
    isAmount = false,
    value = $bindable(""),
    placeholder = "",
    disabled = false,
    error = null,
    autofocus = false,
    class: className = "",
    inputClass = "",
    onkeydown,
    oninput,
    onblur,
  }: Props = $props();

  let isShaking = $state(false);
  let inputRef = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (autofocus && inputRef) {
      inputRef.focus();
    }
  });

  const effectivePlaceholder = $derived(
    placeholder.length > 0 ? placeholder : isAmount ? "0.00" : "",
  );

  export function shake(): void {
    isShaking = true;
    inputRef?.focus();
    setTimeout(() => {
      isShaking = false;
    }, 400);
  }

  export function validate(): boolean {
    const trimmed = value.trim();
    if (required === true && trimmed.length === 0) {
      shake();
      return false;
    }
    if (isAmount === true) {
      const num = Number(trimmed);
      if (Number.isNaN(num) || num <= 0) {
        shake();
        return false;
      }
    }
    return true;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (isAmount === true) {
      const allowedKeys = [
        "Backspace",
        "Delete",
        "Tab",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
        "Enter",
        "Escape",
      ];
      if (allowedKeys.includes(e.key) || e.ctrlKey || e.metaKey) {
        onkeydown?.(e);
        return;
      }
      if (e.key === ".") {
        const target = e.target as HTMLInputElement;
        if (target.value.includes(".")) {
          e.preventDefault();
          return;
        }
        onkeydown?.(e);
        return;
      }
      if (/^[0-9]$/.test(e.key) === false) {
        e.preventDefault();
        return;
      }
    }
    onkeydown?.(e);
  }

  function handleInput(e: Event) {
    if (isAmount === true) {
      const target = e.target as HTMLInputElement;
      let val = target.value.replace(/[^\d.]/g, "");
      const parts = val.split(".");
      if (parts.length > 2) {
        val = parts[0] + "." + parts.slice(1).join("");
      }
      const splitAgain = val.split(".");
      if (splitAgain.length === 2 && splitAgain[1] !== undefined && splitAgain[1].length > 2) {
        val = `${splitAgain[0]}.${splitAgain[1].slice(0, 2)}`;
      }
      value = val;
      target.value = val;
    }
    oninput?.(e);
  }
</script>

{#if label !== undefined && label.length > 0}
  <div class={cn("flex flex-col gap-1.5", isShaking && "animate-shake", className)}>
    <label for={id} class="text-muted-foreground text-xs font-semibold">
      {label}
      {#if required}
        <span class="text-destructive">*</span>
      {/if}
    </label>
    <Input
      {id}
      bind:ref={inputRef}
      bind:value
      {required}
      placeholder={effectivePlaceholder}
      {disabled}
      inputmode={isAmount ? "decimal" : undefined}
      class={cn(
        "border-border/40 bg-background focus-visible:border-foreground/30 focus-visible:ring-1 focus-visible:ring-foreground/20",
        isAmount && "font-mono",
        inputClass,
      )}
      onkeydown={handleKeyDown}
      oninput={handleInput}
      {onblur}
    />
    {#if error !== null && error !== undefined && error.length > 0}
      <p class="text-destructive text-xs font-medium">{error}</p>
    {/if}
  </div>
{:else}
  <Input
    {id}
    bind:ref={inputRef}
    bind:value
    {required}
    placeholder={effectivePlaceholder}
    {disabled}
    inputmode={isAmount ? "decimal" : undefined}
    class={cn(
      "border-border/40 bg-background focus-visible:border-foreground/30 focus-visible:ring-1 focus-visible:ring-foreground/20",
      isAmount && "font-mono",
      className,
      inputClass,
    )}
    onkeydown={handleKeyDown}
    oninput={handleInput}
    {onblur}
  />
{/if}
