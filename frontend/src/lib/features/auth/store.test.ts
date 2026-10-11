import { describe, it, expect, vi, beforeEach } from "vitest";
import { AuthStore } from "./store.svelte";
import { authApi } from "./api";
import { Role, UserId, type UserDto } from "./types";

describe("AuthStore", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes in isInit state and resolves to unauthenticated when 401", async () => {
    vi.spyOn(authApi, "me").mockRejectedValue(new Error("401 Unauthorized"));

    const store = new AuthStore();
    expect(store.isInit).toBe(true);
    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();

    await store.init();
    expect(store.isInit).toBe(false);
    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();
  });

  it("updates requireUser and isAuthenticated on successful login", async () => {
    const mockUser: UserDto = {
      id: UserId("user-123"),
      username: "tester",
      role: Role.Admin,
      createdAt: 1700000000,
    };

    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.login({ username: "tester", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.requireUser()).toEqual(mockUser);
  });

  it("registers user and updates requireUser", async () => {
    const mockUser: UserDto = {
      id: UserId("user-456"),
      username: "newbie",
      role: Role.Member,
      createdAt: 1700000100,
    };

    vi.spyOn(authApi, "register").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.register({ username: "newbie", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.requireUser()).toEqual(mockUser);
  });

  it("rethrows when login fails and remains unauthenticated", async () => {
    vi.spyOn(authApi, "login").mockRejectedValue(new Error("Invalid credentials"));

    const store = new AuthStore();
    await expect(store.login({ username: "bad", password: "pwd" })).rejects.toThrow(
      "Invalid credentials",
    );

    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();
  });

  it("clears user on logout", async () => {
    const mockUser: UserDto = {
      id: UserId("u1"),
      username: "u1",
      role: Role.Member,
      createdAt: 100,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);
    vi.spyOn(authApi, "logout").mockResolvedValue(null);

    const store = new AuthStore();
    await store.login({ username: "u1", password: "pwd" });
    expect(store.isAuthenticated).toBe(true);

    await store.logout();
    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();
  });

  it("requireUser returns UserDto when authenticated or throws InvariantViolationError when unauthenticated", async () => {
    const unauthenticatedStore = new AuthStore();
    vi.spyOn(authApi, "me").mockRejectedValue(new Error("401 Unauthorized"));
    await unauthenticatedStore.init();

    expect(() => unauthenticatedStore.requireUser()).toThrow(
      "User session required. Ensure user is authenticated before accessing current user.",
    );

    const authenticatedStore = new AuthStore();
    const mockUser: UserDto = {
      id: UserId("usr_active"),
      username: "bob",
      role: Role.Admin,
      createdAt: 100,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);
    await authenticatedStore.login({ username: "bob", password: "pwd" });

    const user = authenticatedStore.requireUser();
    expect(user.id).toBe("usr_active");
    expect(user.username).toBe("bob");
    expect(user.role).toBe(Role.Admin);
  });

  it("handles session expiration cleanly, transitions to unauthenticated, and toasts", async () => {
    const store = new AuthStore();
    const mockUser: UserDto = {
      id: UserId("usr_active"),
      username: "alice",
      role: Role.Admin,
      createdAt: 100,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);
    await store.login({ username: "alice", password: "pwd" });
    expect(store.isAuthenticated).toBe(true);

    store.handleSessionExpired("Custom expiration message");

    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();

    // Idempotent: repeated call when already unauthenticated does not throw
    store.handleSessionExpired("Another call");
    expect(store.isAuthenticated).toBe(false);

    store.dispose();
  });
});
