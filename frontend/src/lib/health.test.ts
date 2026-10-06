import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
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
    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(true);
    expect(store.lastChecked).toBeNull();

    await store.check();

    expect(store.isOnline).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.lastChecked).not.toBeNull();
  });

  it("marks service as offline and updates lastChecked when backend returns error code", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 500,
      status: "INTERNAL_ERROR",
      data: null,
    }));

    const store = new HealthStore();
    expect(store.lastChecked).toBeNull();

    await store.check();

    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.lastChecked).not.toBeNull();
  });

  it("marks service as offline and updates lastChecked when network request throws", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => {
      throw new Error("Connection refused");
    });

    const store = new HealthStore();
    expect(store.lastChecked).toBeNull();

    await store.check();

    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.lastChecked).not.toBeNull();
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

  it("exposes state as AsyncState discriminated union and tracks error payload", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => {
      throw new Error("Down for maintenance");
    });

    const store = new HealthStore();
    expect(store.state.status).toBe("loading");

    await store.check();
    expect(store.state.status).toBe("error");
    if (store.state.status === "error") {
      expect(store.state.error.action).toBe("CORE.HEALTH.CHECK_FAILED");
      expect(store.state.error.message).toBe("Down for maintenance");
    }
    expect(store.error).toBe("Down for maintenance");
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
