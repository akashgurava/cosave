import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { api } from "./api";
import { MemoryTransportAdapter } from "./api/testing";
import { HealthStore } from "./health.svelte";

describe("HealthStore and BackendStatusDot reachability", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: (() => void) | undefined;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    vi.useRealTimers();
    if (restoreTransport !== undefined) {
      restoreTransport();
    }
  });

  it("marks service as online when backend health check succeeds", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 0,
      status: "HEALTHY",
      data: {},
    }));

    const store = new HealthStore();
    expect(store.isHealthy).toBe(false);
    expect(store.isLoading).toBe(true);
    expect(store.state.status).toBe("init");

    await store.check();

    expect(store.isHealthy).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.state.status).toBe("healthy");
  });

  it("marks service as offline and updates state when backend returns error code", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 500,
      status: "INTERNAL_ERROR",
      data: null,
    }));

    const store = new HealthStore();
    expect(store.state.status).toBe("init");

    await store.check();

    expect(store.isHealthy).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.state.status).toBe("unreachable");
  });

  it("marks service as offline and updates state when network request throws", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => {
      throw new Error("Connection refused");
    });

    const store = new HealthStore();
    expect(store.state.status).toBe("init");

    await store.check();

    expect(store.isHealthy).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.state.status).toBe("unreachable");
  });

  it("starts periodic polling and cancels cleanly on unsubscribe", async () => {
    vi.useFakeTimers();
    let pingCount = 0;
    memoryTransport.on("GET", "/api/v1/health", () => {
      pingCount++;
      return {
        code: 0,
        status: "HEALTHY",
        data: {},
      };
    });

    const store = new HealthStore();
    const unsubscribe = store.startPolling(5000);

    // Initial check fired immediately
    expect(pingCount).toBe(1);

    // Advance 5 seconds
    await vi.advanceTimersByTimeAsync(5000);
    expect(pingCount).toBe(2);

    // Unsubscribe stops future polling
    unsubscribe();
    await vi.advanceTimersByTimeAsync(5000);
    expect(pingCount).toBe(2);
  });

  it("exposes state as HealthState discriminated union when health check fails", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => {
      throw new Error("Down for maintenance");
    });

    const store = new HealthStore();
    expect(store.state.status).toBe("init");
    expect(store.downDurationSeconds).toBe(-1);

    await store.check();
    expect(store.state.status).toBe("unreachable");
    expect(store.isHealthy).toBe(false);
    expect(store.isUnreachable).toBe(true);
    expect(store.downDurationSeconds).toBeGreaterThanOrEqual(0);
    if (store.state.status === "unreachable") {
      expect(store.state.lastHealthyAt).toBeNull();
    }
  });

  it("stops polling cleanly with stopPolling method", async () => {
    vi.useFakeTimers();
    let pingCount = 0;
    memoryTransport.on("GET", "/api/v1/health", () => {
      pingCount++;
      return { code: 0, status: "HEALTHY", data: {} };
    });

    const store = new HealthStore();
    store.startPolling(10000);
    expect(pingCount).toBe(1);

    store.stopPolling();
    await vi.advanceTimersByTimeAsync(20000);
    expect(pingCount).toBe(1);
  });
});
