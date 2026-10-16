//! Unauthenticated marketing hero section.
//!
//! Features the main value proposition, CTA, and embeds the interactive finance showcase.

use leptos::prelude::*;

use super::finance_showcase::FinanceShowcase;

/// Two-column hero section with headline, CTA, and visual product showcase.
#[component]
pub(crate) fn MarketingHero() -> impl IntoView {
    view! {
        <div class="relative w-full py-8 sm:py-16">
            // Subtle diagonal ambient backdrop lines
            <div
                class="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(16,185,129,0.12),rgba(255,255,255,0))]"
                aria-hidden="true"
            ></div>

            <div class="grid grid-cols-1 items-center gap-12 lg:grid-cols-12 lg:gap-16">
                // Left Column: Copy & Value Proposition
                <div class="flex flex-col items-start text-left lg:col-span-6 xl:col-span-6">
                    // Main Headline
                    <h1 class="text-4xl font-extrabold tracking-tight text-white sm:text-5xl lg:text-6xl lg:leading-tight">
                        "Track Expenses."<br />
                        "Cut Waste."<br />
                        <span class="bg-gradient-to-r from-emerald-400 via-teal-300 to-emerald-500 bg-clip-text text-transparent">
                            "Grow Savings."
                        </span>
                    </h1>

                    // Value Proposition Subhead
                    <p class="mt-5 max-w-lg text-sm leading-relaxed text-white/65 sm:text-base">
                        "A personal and family finance platform built to bring clarity to spend patterns. Track income, reduce everyday expenses, and grow family savings together."
                    </p>

                    // CTA Actions
                    <div class="mt-8 flex flex-wrap items-center gap-4">
                        <button
                            type="button"
                            class="h-12 px-7 rounded-xl bg-white text-xs font-extrabold text-black shadow-lg transition-transform hover:scale-105 hover:bg-neutral-200"
                        >
                            "Get Started Now"
                        </button>
                    </div>

                    // Footer Micro Trust Note
                    <div class="mt-8 flex items-center gap-2 border-t border-white/10 pt-6 text-[11px] text-white/45">
                        <svg
                            class="size-3.5 text-emerald-400"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                        >
                            <polyline points="20 6 9 17 4 12" />
                        </svg>
                        <span>"Spend pattern visibility • Smart expense reduction • Family savings growth"</span>
                    </div>
                </div>

                // Right Column: Visual Product Showcase
                <div class="lg:col-span-6 xl:col-span-6">
                    <FinanceShowcase />
                </div>
            </div>
        </div>
    }
}
