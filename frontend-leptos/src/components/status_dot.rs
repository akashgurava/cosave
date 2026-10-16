//! Backend status beacon component.
//!
//! Provides real-time visual feedback for API health and reachability with
//! an interactive popover and live elapsed time ticker.
//! Mirrors `frontend/src/lib/components/BackendStatusDot.svelte`.

use std::time::Duration;

use gloo_timers::future::sleep;
use leptos::prelude::*;

use crate::health::{HealthContext, HealthStatus};

/// Formats millisecond timestamps into human-readable elapsed duration.
fn format_elapsed_time(now: f64, timestamp: f64) -> String {
    let diff_sec = ((now - timestamp) / 1000.0).floor() as i64;
    let diff_sec = diff_sec.max(0);
    if diff_sec < 5 {
        "just now".to_string()
    } else if diff_sec < 60 {
        format!("{diff_sec}s ago")
    } else {
        let diff_min = diff_sec / 60;
        if diff_min < 60 {
            format!("{diff_min}m ago")
        } else {
            let diff_hour = diff_min / 60;
            format!("{diff_hour}h ago")
        }
    }
}

/// Interactive backend status beacon and connectivity tooltip.
#[component]
pub(crate) fn BackendStatusDot() -> impl IntoView {
    let health = expect_context::<HealthContext>();
    let show_popover = RwSignal::new(false);
    let now = RwSignal::new(js_sys::Date::now());

    // 1-second elapsed time ticker for live popover updates
    leptos::task::spawn_local(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            now.set(js_sys::Date::now());
        }
    });

    let toggle_popover = move |_| {
        show_popover.update(|v| *v = !*v);
    };

    let on_mouse_enter = move |_| {
        show_popover.set(true);
    };

    let on_mouse_leave = move |_| {
        show_popover.set(false);
    };

    let aria_label = move || {
        if health.is_healthy() {
            "Service online"
        } else {
            "Service unavailable"
        }
    };

    view! {
        <div class="relative inline-flex items-center ml-1">
            // Interactive Beacon Button
            <button
                type="button"
                id="backend-status-dot"
                aria-label=aria_label
                aria-expanded=move || show_popover.get().to_string()
                class="group relative flex size-7 cursor-pointer items-center justify-center rounded-full transition-transform hover:scale-110 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500/50"
                on:click=toggle_popover
                on:mouseenter=on_mouse_enter
                on:mouseleave=on_mouse_leave
            >
                {move || match health.status().get() {
                    HealthStatus::Healthy { .. } => view! {
                        <span class="absolute inline-flex size-3.5 animate-ping rounded-full bg-emerald-400 opacity-60" aria-hidden="true"></span>
                        <span class="relative inline-flex size-3 rounded-full bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,0.8)] transition-all group-hover:scale-125" aria-hidden="true"></span>
                    }.into_any(),
                    HealthStatus::Unreachable { .. } => view! {
                        <span class="absolute inline-flex size-3.5 animate-pulse rounded-full bg-rose-500 opacity-50" aria-hidden="true"></span>
                        <span class="relative inline-flex size-3 rounded-full bg-rose-500 shadow-[0_0_12px_rgba(244,63,94,0.8)] transition-all group-hover:scale-125" aria-hidden="true"></span>
                    }.into_any(),
                    HealthStatus::Init => view! {
                        <span class="absolute inline-flex size-3.5 animate-ping rounded-full bg-neutral-400 opacity-40" aria-hidden="true"></span>
                        <span class="relative inline-flex size-3 rounded-full bg-neutral-400 shadow-[0_0_12px_rgba(163,163,163,0.8)] transition-all group-hover:scale-125" aria-hidden="true"></span>
                    }.into_any(),
                }}
            </button>

            // Popover Tooltip
            {move || if show_popover.get() {
                let current_now = now.get();
                let status_dot_class = if health.is_healthy() {
                    "size-2 shrink-0 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"
                } else if health.is_unreachable() {
                    "size-2 shrink-0 rounded-full bg-rose-400 shadow-[0_0_8px_rgba(244,63,94,0.8)]"
                } else {
                    "size-2 shrink-0 rounded-full bg-neutral-400 shadow-[0_0_8px_rgba(163,163,163,0.8)]"
                };

                view! {
                    <div
                        role="tooltip"
                        id="backend-status-popover"
                        class="pointer-events-none absolute top-full left-0 z-50 mt-2 w-48 rounded-xl border border-white/10 bg-[#0a0a0a]/95 p-2.5 shadow-xl backdrop-blur-xl sm:left-1/2 sm:-translate-x-1/2"
                    >
                        <div class="flex items-center gap-2.5">
                            <span class=status_dot_class></span>
                            <div class="min-w-0 flex-1 text-left">
                                {move || match health.status().get() {
                                    HealthStatus::Healthy { checked_at } => view! {
                                        <p class="truncate text-xs font-semibold text-white">"Service Online"</p>
                                        <p class="text-[11px] text-white/60">
                                            {format!("Checked {}", format_elapsed_time(current_now, checked_at))}
                                        </p>
                                    }.into_any(),
                                    HealthStatus::Unreachable { .. } => {
                                        let down_duration = health.down_duration_seconds(current_now).unwrap_or(0);
                                        let last_healthy = health.last_healthy_at();
                                        view! {
                                            <p class="truncate text-xs font-semibold text-rose-400">"Service Unavailable"</p>
                                            <p class="text-[11px] text-rose-300">
                                                {format!("Down for {down_duration}s")}
                                            </p>
                                            {last_healthy.map(|last_ts| view! {
                                                <p class="text-[10px] text-white/45">
                                                    {format!("Last online {}", format_elapsed_time(current_now, last_ts))}
                                                </p>
                                            })}
                                        }.into_any()
                                    },
                                    HealthStatus::Init => view! {
                                        <p class="truncate text-xs font-semibold text-white">"Connecting..."</p>
                                        <p class="text-[11px] text-white/60">"Checking connection..."</p>
                                    }.into_any(),
                                }}
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
