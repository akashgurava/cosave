/**
 * Health check polling store tracking backend reachability and outage duration.
 *
 * Periodically queries `/api/v1/health` to keep the UI's connection beacon
 * and service availability indicators updated in real time.
 */

import { SvelteDate } from "svelte/reactivity";

import { api } from "$lib/api";

/**
 * Health check endpoint payload (empty object).
 */
export type HealthData = Record<string, never>;

/**
 * Current reachability of the backend service.
 *
 * States:
 * - `init`: Waiting for the first health check to run.
 * - `healthy`: Service is online. Contains the timestamp of the last successful check (`checkedAt`).
 * - `unreachable`: Service is offline. Tracks when the outage started (`downSince`) and when the service was last seen online (`lastHealthyAt`).
 */
export type HealthState =
  | { readonly status: "init" }
  | {
      readonly status: "healthy";
      readonly checkedAt: SvelteDate;
    }
  | {
      readonly status: "unreachable";
      readonly downSince: SvelteDate;
      readonly lastHealthyAt: SvelteDate | null;
    };

/**
 * Reactive store tracking backend service reachability via polling.
 */
export class HealthStore {
  #state = $state<HealthState>({ status: "init" });
  #isChecking = false;

  #activeTimer: ReturnType<typeof setInterval> | null = null;

  /**
   * Current authoritative health state discriminated union.
   */
  public get state(): HealthState {
    return this.#state;
  }

  /**
   * True strictly when the backend is online and responding healthy.
   */
  public get isHealthy(): boolean {
    return this.#state.status === "healthy";
  }

  /**
   * True strictly when the backend is unreachable or returning failure.
   */
  public get isUnreachable(): boolean {
    return this.#state.status === "unreachable";
  }

  /**
   * Timestamp of the most recent successful health check, or null if uninitialized/unreachable.
   */
  public get checkedAt(): SvelteDate | null {
    if (this.#state.status === "healthy") {
      return this.#state.checkedAt;
    }
    return null;
  }

  /**
   * Timestamp when the service was last seen online, or null if never seen online.
   */
  public get lastHealthyAt(): SvelteDate | null {
    if (this.#state.status === "healthy") {
      return this.#state.checkedAt;
    }
    if (this.#state.status === "unreachable") {
      return this.#state.lastHealthyAt;
    }
    return null;
  }

  /**
   * True strictly before the initial health check has executed.
   */
  public get isInit(): boolean {
    return this.#state.status === "init";
  }

  /**
   * True when a check is in-flight or the store is uninitialized.
   */
  public get isLoading(): boolean {
    return this.#isChecking === true || this.#state.status === "init";
  }

  /**
   * Elapsed outage duration in whole seconds.
   *
   * # Returns
   * - Positive whole seconds (`>= 0`) elapsed since `downSince` if currently unreachable.
   * - `-1` sentinel if the service is healthy or initializing.
   */
  public get downDurationSeconds(): number {
    if (this.#state.status !== "unreachable") {
      return -1;
    }
    return Math.max(0, Math.floor((Date.now() - this.#state.downSince.getTime()) / 1000));
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
    const now = new SvelteDate();
    try {
      await api.get<HealthData>("/api/v1/health");
      this.#state = { status: "healthy", checkedAt: now };
    } catch {
      let lastHealthyAt: SvelteDate | null = null;
      if (this.#state.status === "healthy") {
        lastHealthyAt = this.#state.checkedAt;
      } else if (this.#state.status === "unreachable") {
        lastHealthyAt = this.#state.lastHealthyAt;
      }

      let downSince = now;
      if (this.#state.status === "unreachable") {
        downSince = this.#state.downSince;
      }

      this.#state = {
        status: "unreachable",
        downSince,
        lastHealthyAt,
      };
    } finally {
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
