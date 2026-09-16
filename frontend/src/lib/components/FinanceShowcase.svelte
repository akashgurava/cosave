<script lang="ts">
  const months = [
    { label: "Feb", height: "45%", active: false },
    { label: "Mar", height: "65%", active: false },
    { label: "Apr", height: "90%", active: true },
    { label: "May", height: "100%", active: false },
    { label: "Jun", height: "70%", active: false },
    { label: "Jul", height: "55%", active: false },
  ];
</script>

<div
  class="relative mx-auto flex w-full max-w-lg items-center justify-center py-6 select-none sm:py-10"
>
  <!-- Subtle ambient gradient glow background -->
  <div
    class="pointer-events-none absolute -inset-4 rounded-full bg-emerald-500/10 blur-3xl"
    aria-hidden="true"
  ></div>

  <div class="relative w-full">
    <!-- 1. Floating Savings Tag (Top Left behind) -->
    <div
      class="absolute -top-3 left-4 z-10 flex items-center gap-2.5 rounded-2xl border border-(--border-subtle) bg-(--bg-surface)/90 px-3.5 py-2 shadow-lg backdrop-blur-md transition-transform hover:-translate-y-0.5"
    >
      <div
        class="flex size-7 items-center justify-center rounded-lg bg-emerald-500/15 text-emerald-400"
      >
        <svg
          class="size-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <polyline points="23 6 13.5 15.5 8.5 10.5 1 18" />
          <polyline points="17 6 23 6 23 12" />
        </svg>
      </div>
      <div>
        <p class="text-[11px] font-semibold text-(--text-primary)">Monthly Savings</p>
        <p class="text-[9px] font-semibold text-emerald-400">$1,250 saved this month</p>
      </div>
    </div>

    <!-- 2. Payment / Account Card (Top Right floating) -->
    <div
      class="absolute -top-6 -right-2 z-20 w-52 rounded-2xl border border-amber-500/30 bg-linear-to-br from-amber-400 via-amber-500 to-amber-600 p-4 text-black shadow-2xl transition-transform hover:scale-105 sm:-right-4 sm:w-56 sm:p-5"
    >
      <!-- Card Header: Mastercard rings + Contactless -->
      <div class="flex items-center justify-between">
        <div class="flex -space-x-2">
          <div class="size-6 rounded-full bg-black/40"></div>
          <div class="size-6 rounded-full bg-black/25"></div>
        </div>
        <svg
          class="size-5 text-black/70"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
        >
          <path d="M8.5 16.5a5 5 0 0 1 0-7" />
          <path d="M11.5 18.5a8 8 0 0 1 0-11" />
          <path d="M14.5 20.5a11 11 0 0 1 0-15" />
        </svg>
      </div>

      <!-- Card Balance -->
      <div class="mt-4 sm:mt-5">
        <p class="text-[10px] font-medium tracking-wide text-black/75 uppercase">Family Balance</p>
        <p class="text-xl font-black tracking-tight text-black sm:text-2xl">$14,850.75</p>
      </div>

      <!-- Card Footer: Masked number -->
      <div class="mt-3 flex items-center justify-between text-[11px] font-semibold text-black/80">
        <span>•••• 4289</span>
        <span class="text-[9px] tracking-wider uppercase">Shared Account</span>
      </div>
    </div>

    <!-- 3. Income / Analytics Glass Card (Centerpiece) -->
    <div
      class="relative z-30 mx-auto mt-12 w-full max-w-sm rounded-3xl border border-(--border-subtle) bg-(--bg-glass) p-5 shadow-2xl backdrop-blur-2xl transition-all sm:mt-14 sm:p-6"
    >
      <!-- Card Top Bar -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="size-2 rounded-full bg-emerald-400"></div>
          <span class="text-xs font-semibold text-(--text-primary)">Monthly Inflow & Spend</span>
        </div>

        <!-- M / Y Toggle -->
        <div
          class="flex items-center gap-1 rounded-lg border border-(--border-subtle) bg-(--bg-hover) p-0.5 text-[10px]"
        >
          <span class="rounded bg-(--bg-surface) px-1.5 py-0.5 font-bold text-(--text-primary)"
            >M</span
          >
          <span class="px-1.5 py-0.5 text-(--text-muted)">Y</span>
        </div>
      </div>

      <!-- Bar Chart Visualization -->
      <div class="mt-6 flex h-28 items-end justify-between gap-2.5 px-2 sm:h-32">
        {#each months as m (m.label)}
          <div class="flex flex-1 flex-col items-center gap-2">
            <div class="relative flex h-20 w-full items-end justify-center sm:h-24">
              <div
                class="w-full rounded-full transition-all duration-500 {m.active
                  ? 'bg-emerald-500 shadow-md shadow-emerald-500/30'
                  : 'bg-(--text-primary) opacity-20 hover:opacity-30'}"
                style:height={m.height}
              ></div>
            </div>
            <span
              class="text-[10px] font-medium {m.active
                ? 'font-bold text-emerald-400'
                : 'text-(--text-muted)'}"
            >
              {m.label}
            </span>
          </div>
        {/each}
      </div>

      <!-- Mini Nav / Tool Dock -->
      <div
        class="mt-5 flex items-center justify-around rounded-2xl border border-(--border-subtle) bg-(--bg-hover) p-1.5 text-(--text-secondary)"
      >
        <button
          type="button"
          class="flex size-7 cursor-pointer items-center justify-center rounded-xl transition-colors hover:text-(--text-primary)"
          aria-label="Overview"
        >
          <svg
            class="size-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M21.21 15.89A10 10 0 1 1 8 2.83" />
            <path d="M22 12A10 10 0 0 0 12 2v10z" />
          </svg>
        </button>

        <button
          type="button"
          class="flex size-7 items-center justify-center rounded-xl bg-(--bg-surface) text-(--text-primary) shadow-sm"
          aria-label="Analytics"
        >
          <svg
            class="size-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
          </svg>
        </button>

        <button
          type="button"
          class="flex size-7 cursor-pointer items-center justify-center rounded-xl transition-colors hover:text-(--text-primary)"
          aria-label="Savings"
        >
          <svg
            class="size-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <line x1="12" y1="1" x2="12" y2="23" />
            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
          </svg>
        </button>

        <button
          type="button"
          class="flex size-7 cursor-pointer items-center justify-center rounded-xl transition-colors hover:text-(--text-primary)"
          aria-label="Family Members"
        >
          <svg
            class="size-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
            <circle cx="9" cy="7" r="4" />
            <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 4. Floating Transaction Pill (Bottom Right) -->
    <div
      class="absolute right-1 -bottom-4 z-40 flex items-center gap-3 rounded-2xl border border-(--border-subtle) bg-(--bg-surface)/95 px-4 py-2.5 shadow-xl backdrop-blur-xl transition-transform hover:scale-105 sm:right-2 sm:-bottom-5"
    >
      <div class="flex size-8 items-center justify-center rounded-xl bg-pink-500/15 text-pink-400">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z" />
          <path d="M3 6h18" />
          <path d="M16 10a4 4 0 0 1-8 0" />
        </svg>
      </div>
      <div>
        <p class="text-xs font-semibold text-(--text-primary)">Family Grocery</p>
        <p class="text-[10px] text-(--text-secondary)">Spend pattern: -12% vs last mo</p>
      </div>
      <span class="ml-2 text-xs font-bold text-rose-400">-$42.80</span>
    </div>
  </div>
</div>
