//! Backend status beacon component.
//!
//! Provides visual feedback for API health and connectivity status.

use leptos::prelude::*;

/// Pulsing status beacon indicating server availability.
#[component]
pub fn BackendStatusDot() -> impl IntoView {
    view! {
        <div class="relative inline-flex items-center ml-1">
            <div class="group relative flex size-7 cursor-pointer items-center justify-center rounded-full">
                <span class="absolute inline-flex size-3.5 animate-ping rounded-full bg-emerald-400 opacity-60" aria-hidden="true"></span>
                <span class="relative inline-flex size-3 rounded-full bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,0.8)]" aria-hidden="true"></span>
            </div>
        </div>
    }
}
