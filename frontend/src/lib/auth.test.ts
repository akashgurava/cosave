import { describe, it, expect, vi, beforeEach } from "vitest";
import { AuthStore } from "./auth";
import { authApi, Code, Status, type UserDto } from "./api";

describe("AuthStore", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes in loading state and resolves to null if unauthenticated", async () => {
    vi.spyOn(authApi, "me").mockRejectedValue(new Error("401 Unauthorized"));

    const store = new AuthStore();
    expect(store.isAuthenticated).toBe(false);
    expect(store.currentUser).toBeNull();

    await store.init();
    expect(store.isLoading).toBe(false);
    expect(store.isAuthenticated).toBe(false);
    expect(store.currentUser).toBeNull();
  });

  it("updates currentUser and isAuthenticated on successful login", async () => {
    const mockUser: UserDto = {
      id: "user-123",
      name: "tester",
      role: "admin",
      created_at: 1700000000,
    };

    vi.spyOn(authApi, "login").mockResolvedValue({
      code: Code.Zero,
      status: Status.Ok,
      data: mockUser,
    });

    const store = new AuthStore();
    await store.login({ name: "tester", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.currentUser).toEqual(mockUser);
    expect(store.error).toBeNull();
  });

  it("clears currentUser on logout", async () => {
    vi.spyOn(authApi, "logout").mockResolvedValue({
      code: Code.Zero,
      status: Status.Ok,
      data: null,
    });

    const store = new AuthStore();
    store.currentUser = {
      id: "u1",
      name: "u1",
      role: "member",
      created_at: 100,
    };
    expect(store.isAuthenticated).toBe(true);

    await store.logout();
    expect(store.isAuthenticated).toBe(false);
    expect(store.currentUser).toBeNull();
  });
});
