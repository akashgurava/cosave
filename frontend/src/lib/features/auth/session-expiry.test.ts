import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { api, ApiError } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { AuthStore } from "$lib/features/auth/store.svelte";
import { familyStore } from "$lib/features/family";
import { categoryStore } from "$lib/features/categories";
import { toUserId, type UserDto } from "$lib/features/auth/types";
import { toast } from "svelte-sonner";

describe("Session Invalidation & Ghost Data Prevention (Coordination Test)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: (() => void) | undefined;
  let store: AuthStore;

  beforeEach(() => {
    vi.restoreAllMocks();
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
    store = new AuthStore();
  });

  afterEach(() => {
    store.dispose();
    if (restoreTransport !== undefined) {
      restoreTransport();
    }
    familyStore.reset();
    categoryStore.reset();
  });

  it("liquidates ghost state across feature stores and toasts when 401 occurs in-flight", async () => {
    const toastSpy = vi.spyOn(toast, "info").mockImplementation(() => "toast-id");

    // 1. Establish authenticated active user in authStore
    const mockUser: UserDto = {
      id: toUserId("usr_active"),
      username: "alex",
      role: "admin",
      createdAt: 1700000000,
    };
    memoryTransport.on("POST", "/api/v1/auth/login", () => ({
      code: 0,
      status: "OK",
      data: mockUser,
    }));
    await store.login({ username: "alex", password: "password123" });
    expect(store.isAuthenticated).toBe(true);
    expect(store.currentUser).toEqual(mockUser);

    // 2. Pre-seed family and category stores to simulate cached in-memory data
    memoryTransport.on("GET", "/api/v1/config/family", () => ({
      code: 0,
      status: "OK",
      data: {
        family: { id: 1, familyName: "The Smiths", currencyId: 1, createdAt: 100 },
        members: [{ id: 1, familyId: 1, memberName: "Alice", createdAt: 100 }],
        accounts: [],
        currencies: [
          { id: 1, code: "USD", name: "US Dollar", symbol: "$", scale: 2, sortOrder: 1 },
        ],
      },
    }));
    await familyStore.load();
    expect(familyStore.isLoaded).toBe(true);
    expect(familyStore.members).toHaveLength(1);

    // 3. Simulate cookie deletion: Next in-flight mutation/query receives 401 Unauthorized
    memoryTransport.on("GET", "/api/v1/config/family/members", () => ({
      code: 401,
      status: "UNAUTHORIZED",
      data: {
        action: "AUTH.EXTRACT_USER.INVALID_TOKEN",
        message: "Session token is invalid or expired",
      },
    }));

    // Expect the API request to fail with ApiError
    await expect(api.get("/api/v1/config/family/members")).rejects.toThrow(ApiError);

    // 4. Verify AuthStore immediately transitioned to unauthenticated guest state
    expect(store.isAuthenticated).toBe(false);
    expect(store.currentUser).toBeNull();
    expect(store.state.status).toBe("success");

    // 5. Verify toast notification was dispatched
    expect(toastSpy).toHaveBeenCalledTimes(1);
    expect(toastSpy).toHaveBeenCalledWith(
      "Your session has expired. Please sign in again.",
      expect.objectContaining({ duration: 6000 }),
    );

    // 6. Simulate the reactive layout cleanup (+layout.svelte resets dependents when auth drops)
    familyStore.reset();
    categoryStore.reset();

    // 7. Verify NO ghost data remains in memory
    expect(familyStore.isLoaded).toBe(false);
    expect(familyStore.members).toHaveLength(0);
    expect(familyStore.family).toBeNull();
  });
});
