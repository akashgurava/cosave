import { apiFetch, Code, Status, type ApiResponse } from "$lib/api";
import { SvelteDate } from "svelte/reactivity";

/**
 * Health check endpoint payload.
 */
export interface HealthData {
  service: string;
}

/**
 * Reactive store tracking backend service reachability via polling.
 */
export class HealthStore {
  public health = $state<ApiResponse<HealthData> | null>(null);
  public isLoading = $state<boolean>(true);
  public lastChecked = $state<SvelteDate | null>(null);

  /**
   * True if the latest health ping responded with code 0 and HEALTHY status.
   */
  public isOnline = $derived<boolean>(
    this.health !== null && this.health.code === Code.Zero && this.health.status === Status.Healthy,
  );

  private activeTimer: ReturnType<typeof setInterval> | null = null;

  /**
   * Pings `/api/v1/health` and updates local state.
   */
  public async check(): Promise<void> {
    try {
      this.health = await apiFetch<HealthData>("/api/v1/health");
      this.lastChecked = new SvelteDate();
    } catch {
      this.health = null;
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Starts periodic polling of the health endpoint. Returns an unsubscribe cleanup function.
   */
  public startPolling(intervalMs = 30000): () => void {
    if (this.activeTimer !== null) {
      clearInterval(this.activeTimer);
      this.activeTimer = null;
    }
    this.check();
    this.activeTimer = setInterval(() => {
      this.check();
    }, intervalMs);
    return () => {
      if (this.activeTimer !== null) {
        clearInterval(this.activeTimer);
        this.activeTimer = null;
      }
    };
  }
}

export const healthStore = new HealthStore();
