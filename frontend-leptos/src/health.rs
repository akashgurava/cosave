//! Backend health monitoring and reachability service.
//!
//! Periodically queries `/api/v1/health` to keep the UI's connection beacon
//! and service availability indicators updated in real time.
//! Mirrors the behavior and state machine of Svelte's `frontend/src/lib/health.svelte.ts`.

use std::time::Duration;

use gloo_net::http::Request;
use gloo_timers::future::sleep;
use leptos::prelude::*;
use serde::Deserialize;

use crate::api::ApiResponse;

/// Standard health check payload returned inside `ApiResponse<HealthData>`.
#[derive(Clone, Debug, Deserialize)]
struct HealthData {}

/// Reachability state of the backend service.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HealthStatus {
    /// Initializing; waiting for the first health check to complete.
    Init,
    /// Service is online and responsive. Contains epoch millisecond timestamp of last check.
    Healthy { checked_at: f64 },
    /// Service is offline or unreachable. Tracks when outage started and when last online.
    Unreachable {
        down_since: f64,
        last_healthy_at: Option<f64>,
    },
}

/// Reactive health store holding state signals and check actions.
#[derive(Clone, Copy)]
pub(crate) struct HealthContext {
    status: ReadSignal<HealthStatus>,
    is_checking: ReadSignal<bool>,
    set_status: WriteSignal<HealthStatus>,
    set_is_checking: WriteSignal<bool>,
}

impl Default for HealthContext {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthContext {
    /// Creates a new health context and initializes reactive signals.
    pub(crate) fn new() -> Self {
        let (status, set_status) = signal(HealthStatus::Init);
        let (is_checking, set_is_checking) = signal(false);

        Self {
            status,
            is_checking,
            set_status,
            set_is_checking,
        }
    }

    /// Read-only accessor for the current health status signal.
    pub(crate) fn status(&self) -> ReadSignal<HealthStatus> {
        self.status
    }

    /// Returns `true` strictly when the backend is online and responding.
    pub(crate) fn is_healthy(&self) -> bool {
        matches!(self.status.get(), HealthStatus::Healthy { .. })
    }

    /// Returns `true` strictly when the backend is unreachable or returning errors.
    pub(crate) fn is_unreachable(&self) -> bool {
        matches!(self.status.get(), HealthStatus::Unreachable { .. })
    }

    /// Returns the timestamp when the service was last verified healthy, if known.
    pub(crate) fn last_healthy_at(&self) -> Option<f64> {
        match self.status.get() {
            HealthStatus::Healthy { checked_at } => Some(checked_at),
            HealthStatus::Unreachable {
                last_healthy_at, ..
            } => last_healthy_at,
            HealthStatus::Init => None,
        }
    }

    /// Computes the elapsed outage duration in whole seconds.
    pub(crate) fn down_duration_seconds(&self, now: f64) -> Option<i64> {
        match self.status.get() {
            HealthStatus::Unreachable { down_since, .. } => {
                let diff_secs = ((now - down_since) / 1000.0).floor() as i64;
                Some(diff_secs.max(0))
            }
            _ => None,
        }
    }

    /// Performs an asynchronous health check ping against `/api/v1/health`.
    pub(crate) async fn check(&self) {
        if self.is_checking.get_untracked() {
            return;
        }
        self.set_is_checking.set(true);

        let now = js_sys::Date::now();
        let result = Request::get("/api/v1/health").send().await;

        let is_ok = match result {
            Ok(resp) if resp.ok() => match resp.json::<ApiResponse<HealthData>>().await {
                Ok(payload) => {
                    let _ = payload.data();
                    payload.is_ok() && payload.code() == 0 && payload.status() == "HEALTHY"
                }
                Err(_) => false,
            },
            _ => false,
        };

        if is_ok {
            self.set_status
                .set(HealthStatus::Healthy { checked_at: now });
        } else {
            let current = self.status.get_untracked();
            let last_healthy = match current {
                HealthStatus::Healthy { checked_at } => Some(checked_at),
                HealthStatus::Unreachable {
                    last_healthy_at, ..
                } => last_healthy_at,
                HealthStatus::Init => None,
            };

            let down_since = match current {
                HealthStatus::Unreachable { down_since, .. } => down_since,
                _ => now,
            };

            self.set_status.set(HealthStatus::Unreachable {
                down_since,
                last_healthy_at: last_healthy,
            });
        }

        self.set_is_checking.set(false);
    }

    /// Spawns the background 30-second polling task.
    pub(crate) fn start_polling(&self) {
        let ctx = *self;
        leptos::task::spawn_local(async move {
            loop {
                ctx.check().await;
                sleep(Duration::from_secs(30)).await;
            }
        });
    }
}
