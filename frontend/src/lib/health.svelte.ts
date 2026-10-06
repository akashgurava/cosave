/**
 * Health check polling store tracking backend reachability.
 *
 * Periodically queries `/api/v1/health` to keep the UI's connection status dot
 * and service availability indicators updated in real time.
 */

import { api, type ErrorPayload } from "$lib/api";
import type { AsyncState } from "$lib/types";
import { SvelteDate } from "svelte/reactivity";

/**
 * Health check endpoint payload (empty object).
 */
export type HealthData = Record<string, never>;

/**
 * Reactive store tracking backend service reachability via polling.
 */
export class HealthStore {
  #state = $state<AsyncState<HealthData>>({ status: "loading" });
  #lastChecked = $state<SvelteDate | null>(null);
  #isChecking = false;

  #activeTimer: ReturnType<typeof setInterval> | null = null;

  public get state(): AsyncState<HealthData> {
    return this.#state;
  }

  public get isOnline(): boolean {
    return this.#state.status === "success";
  }

  public get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  public get lastChecked(): SvelteDate | null {
    return this.#lastChecked;
  }

  public get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  /**
   * Pings `/api/v1/health` and updates local state.
   * Deduplicates concurrent check calls.
   */
  public async check(): Promise<void> {
    if (this.#isChecking === true) {
      return;
    }
    this.#isChecking = true;
    try {
      const data = await api.get<HealthData>("/api/v1/health");
      this.#state = { status: "success", data };
    } catch (err: unknown) {
      const error: ErrorPayload = {
        action: "CORE.HEALTH.CHECK_FAILED",
        message: err instanceof Error ? err.message : "Service unavailable",
      };
      this.#state = { status: "error", error };
    } finally {
      this.#lastChecked = new SvelteDate();
      this.#isChecking = false;
    }
  }

  /**
   * Stops active polling timer if one exists.
   */
  public stopPolling(): void {
    if (this.#activeTimer !== null) {
      clearInterval(this.#activeTimer);
      this.#activeTimer = null;
    }
  }

  /**
   * Starts periodic polling of the health endpoint. Returns an unsubscribe cleanup function.
   */
  public startPolling(intervalMs = 30000): () => void {
    this.stopPolling();
    void this.check();
    this.#activeTimer = setInterval(() => {
      void this.check();
    }, intervalMs);

    return () => {
      this.stopPolling();
    };
  }
}

export const healthStore = new HealthStore();
