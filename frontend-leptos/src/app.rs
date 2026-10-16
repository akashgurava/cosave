//! Root application shell component.
//!
//! Mirrors `frontend/src/routes/+layout.svelte` and `+page.svelte`.

use leptos::prelude::*;

use crate::components::unauth::{MarketingHero, TopNav};
use crate::health::HealthContext;

/// Root component that wraps the layout and active page views.
#[component]
pub(crate) fn App() -> impl IntoView {
    let health = HealthContext::new();
    provide_context(health);
    health.start_polling();

    view! {
        <div class="bg-black text-white flex min-h-screen flex-col font-sans selection:bg-emerald-500/20 selection:text-emerald-300">
            <TopNav />
            <main class="mx-auto flex w-full max-w-6xl flex-1 flex-col p-6 sm:py-10">
                <MarketingHero />
            </main>
        </div>
    }
}
