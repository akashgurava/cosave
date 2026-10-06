import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { api } from "./api";
import { MemoryTransportAdapter } from "./api/testing";
import { HealthStore } from "./health.svelte";

describe("HealthStore and BackendStatusDot reachability", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    vi.useRealTimers();
    restoreTransport?.();
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
});
