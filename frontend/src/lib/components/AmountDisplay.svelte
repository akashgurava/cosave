<script lang="ts">
  import type { Currency, MinorUnits } from "$lib/types/core";

  interface Props {
    amount: MinorUnits;
    currency: Currency;
    color?: string;
    class?: string;
  }

  let { amount, currency, color, class: className = "" }: Props = $props();

  const major = $derived(
    currency.scale === 0 ? Math.abs(amount) : Math.abs(amount) / 10 ** currency.scale,
  );

  const formattedValue = $derived.by(() => {
    try {
      return new Intl.NumberFormat(undefined, {
        style: "currency",
        currency: currency.code,
        minimumFractionDigits: currency.scale,
        maximumFractionDigits: currency.scale,
      }).format(major);
    } catch {
      return `${currency.code} ${major.toFixed(currency.scale)}`;
    }
  });
</script>

<span
  class="inline-flex items-center font-mono whitespace-nowrap tabular-nums {className}"
  style={color !== undefined && color.trim().length > 0 ? `color: ${color.trim()};` : undefined}
>
  {formattedValue}
</span>
