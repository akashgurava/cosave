//! Unauthenticated top navigation bar component.
//!
//! Provides brand identity, system health status dot, and sign-in trigger.

use leptos::prelude::*;

use crate::components::status_dot::BackendStatusDot;

/// Sticky top navigation bar for the unauthenticated landing view.
#[component]
pub(crate) fn TopNav() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-40 w-full border-b border-white/10 bg-black/80 px-6 py-3.5 backdrop-blur-md transition-colors">
            <div class="mx-auto flex max-w-6xl items-center justify-between">
                // Left: Brand & Server Status Beacon
                <div class="flex items-center gap-2.5">
                    <a href="/" class="flex items-center gap-2">
                        <div class="flex size-7 items-center justify-center rounded-lg bg-gradient-to-tr from-emerald-600 via-emerald-500 to-teal-400 text-white shadow-sm">
                            <svg
                                class="size-3.5"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                aria-hidden="true"
                            >
                                <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                            </svg>
                        </div>
                        <span class="text-sm font-bold tracking-tight text-white">"CoSave"</span>
                    </a>

                    // Backend Status Beacon
                    <BackendStatusDot />
                </div>

                // Right: Authentication Action
                <div class="flex items-center gap-3">
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-xs font-bold text-black shadow-md transition-colors hover:bg-neutral-200"
                    >
                        "Sign In"
                    </button>
                </div>
            </div>
        </header>
    }
}
