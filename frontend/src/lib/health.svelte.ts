/**
 * Health check polling store tracking backend reachability.
 *
 * Periodically queries `/api/v1/health` to keep the UI's connection status dot
 * and service availability indicators updated in real time.
 */

import { api } from "$lib/api";
import { SvelteDate } from "svelte/reactivity";

/**
 * Health check endpoint payload (empty object).
 */
export type HealthData = Record<string, never>;

/**
 * Reactive store tracking backend service reachability via polling.
 */
export class HealthStore {
  #isOnline = $state<boolean>(false);
  #isLoading = $state<boolean>(true);
  #lastChecked = $state<SvelteDate | null>(null);

  #activeTimer: ReturnType<typeof setInterval> | null = null;

  public get isOnline(): boolean {
    return this.#isOnline;
  }

  public get isLoading(): boolean {
    return this.#isLoading;
  }

  public get lastChecked(): SvelteDate | null {
    return this.#lastChecked;
  }

  /**
   * Pings `/api/v1/health` and updates local state.
   */
  public async check(): Promise<void> {
    try {
      await api.get<HealthData>("/api/v1/health");
      this.#isOnline = true;
    } catch {
      this.#isOnline = false;
    } finally {
      this.#lastChecked = new SvelteDate();
      this.#isLoading = false;
    }
  }

  /**
   * Starts periodic polling of the health endpoint. Returns an unsubscribe cleanup function.
   */
  public startPolling(intervalMs = 30000): () => void {
    if (this.#activeTimer !== null) {
      clearInterval(this.#activeTimer);
      this.#activeTimer = null;
    }
    this.check();
    this.#activeTimer = setInterval(() => {
      this.check();
    }, intervalMs);

    return () => {
      if (this.#activeTimer !== null) {
        clearInterval(this.#activeTimer);
        this.#activeTimer = null;
      }
    };
  }
}

export const healthStore = new HealthStore();
