<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import FinanceShowcase from "./FinanceShowcase.svelte";

  interface Props {
    onOpenAuth: () => void;
  }

  let { onOpenAuth }: Props = $props();
</script>

<div class="relative w-full py-8 sm:py-16">
  <!-- Subtle diagonal ambient backdrop lines -->
  <div
    class="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(16,185,129,0.1),rgba(255,255,255,0))]"
    aria-hidden="true"
  ></div>

  <div class="grid grid-cols-1 items-center gap-12 lg:grid-cols-12 lg:gap-16">
    <!-- Left Column: Copy & Value Proposition -->
    <div class="flex flex-col items-start text-left lg:col-span-6 xl:col-span-6">
      <!-- Main Headline -->
      <h1
        class="text-4xl font-extrabold tracking-tight text-(--text-primary) sm:text-5xl lg:text-6xl lg:leading-tight"
      >
        Track Expenses.<br />
        Cut Waste.<br />
        <span class="bg-(image:--brand-gradient) bg-clip-text text-transparent">
          Grow Savings.
        </span>
      </h1>

      <!-- Value Proposition Subhead -->
      <p class="mt-5 max-w-lg text-sm leading-relaxed text-(--text-secondary) sm:text-base">
        A personal and family finance platform built to bring clarity to spend patterns. Track
        income, reduce everyday expenses, and grow family savings together.
      </p>

      <!-- CTA Actions -->
      <div class="mt-8 flex flex-wrap items-center gap-4">
        <Button size="lg" onclick={onOpenAuth} class="h-12 px-7 text-xs font-extrabold shadow-lg">
          Get Started Now
        </Button>
      </div>

      <!-- Footer Micro Trust Note -->
      <div
        class="mt-8 flex items-center gap-2 border-t border-(--border-subtle) pt-6 text-[11px] text-(--text-muted)"
      >
        <svg
          class="size-3.5 text-emerald-400"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <span>Spend pattern visibility • Smart expense reduction • Family savings growth</span>
      </div>
    </div>

    <!-- Right Column: Visual Product Showcase -->
    <div class="lg:col-span-6 xl:col-span-6">
      <FinanceShowcase />
    </div>
  </div>
</div>
