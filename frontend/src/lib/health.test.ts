import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { api, MemoryTransportAdapter, Status } from "./api";
import { HealthStore } from "./health";

describe("HealthStore and BackendStatusDot reachability", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  it("marks service as online when backend health check returns HEALTHY status", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 0,
      status: "HEALTHY",
      data: {},
    }));

    const store = new HealthStore();
    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(true);

    await store.check();

    expect(store.health).not.toBeNull();
    expect(store.health?.status).toBe(Status.Healthy);
    expect(store.isOnline).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.lastChecked).not.toBeNull();
  });

  it("marks service as online when backend health check returns OK status", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 0,
      status: "OK",
      data: {},
    }));

    const store = new HealthStore();
    await store.check();

    expect(store.health).not.toBeNull();
    expect(store.health?.status).toBe(Status.Ok);
    expect(store.isOnline).toBe(true);
  });

  it("marks service as offline when backend returns error code", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => ({
      code: 500,
      status: "INTERNAL_ERROR",
      data: null,
    }));

    const store = new HealthStore();
    await store.check();

    expect(store.health).toBeNull();
    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(false);
  });

  it("marks service as offline when network request throws", async () => {
    memoryTransport.on("GET", "/api/v1/health", () => {
      throw new Error("Connection refused");
    });

    const store = new HealthStore();
    await store.check();

    expect(store.health).toBeNull();
    expect(store.isOnline).toBe(false);
    expect(store.isLoading).toBe(false);
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

    vi.useRealTimers();
  });
});
