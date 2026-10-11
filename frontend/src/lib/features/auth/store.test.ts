import { describe, it, expect, vi, beforeEach } from "vitest";
import { AuthStore } from "./store.svelte";
import { authApi } from "./api";
import { Role, UserId, type UserDto } from "./types";

describe("AuthStore", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("initializes in loading state and resolves to null if unauthenticated", async () => {
    vi.spyOn(authApi, "me").mockRejectedValue(new Error("401 Unauthorized"));

    const store = new AuthStore();
    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();

    await store.init();
    expect(store.isLoading).toBe(false);
    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();
  });

  it("updates requireUser and isAuthenticated on successful login", async () => {
    const mockUser: UserDto = {
      id: UserId("user-123"),
      username: "tester",
      role: "admin",
      createdAt: 1700000000,
    };

    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.login({ username: "tester", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.requireUser()).toEqual(mockUser);
    expect(store.error).toBeNull();
  });

  it("registers user and updates requireUser", async () => {
    const mockUser: UserDto = {
      id: UserId("user-456"),
      username: "newbie",
      role: "member",
      createdAt: 1700000100,
    };

    vi.spyOn(authApi, "register").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.register({ username: "newbie", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.requireUser()).toEqual(mockUser);
    expect(store.error).toBeNull();
  });

  it("sets error and rethrows when login fails", async () => {
    vi.spyOn(authApi, "login").mockRejectedValue(new Error("Invalid credentials"));

    const store = new AuthStore();
    await expect(store.login({ username: "bad", password: "pwd" })).rejects.toThrow(
      "Invalid credentials",
    );

    expect(store.error).toBe("Invalid credentials");
    expect(store.isAuthenticated).toBe(false);
  });

  it("clears user on logout", async () => {
    const mockUser: UserDto = {
      id: UserId("u1"),
      username: "u1",
      role: "member",
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
    expect(store.error).toBeNull();
  });

  it("exposes state as AsyncState discriminated union and correctly derives isAdmin", async () => {
    const adminUser: UserDto = {
      id: UserId("u-admin"),
      username: "super",
      role: Role.Admin,
      createdAt: 200,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(adminUser);

    const store = new AuthStore();
    expect(store.state.status).toBe("loading");

    await store.login({ username: "super", password: "pwd" });
    expect(store.state.status).toBe("success");
    if (store.state.status === "success") {
      expect(store.state.data).toEqual(adminUser);
    }
    expect(store.isAdmin).toBe(true);

    const memberUser: UserDto = {
      id: UserId("u-member"),
      username: "regular",
      role: Role.Member,
      createdAt: 201,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(memberUser);
    await store.login({ username: "regular", password: "pwd" });
    expect(store.isAdmin).toBe(false);
  });

  it("exposes isSuccess accessor matching state status", async () => {
    const store = new AuthStore();
    expect(store.isSuccess).toBe(false);

    vi.spyOn(authApi, "login").mockResolvedValue({
      id: UserId("usr_1"),
      username: "alex",
      role: "member",
      createdAt: 100,
    });
    await store.login({ username: "alex", password: "pwd" });

    expect(store.isSuccess).toBe(true);
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
      role: "admin",
      createdAt: 100,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);
    await authenticatedStore.login({ username: "bob", password: "pwd" });

    const user = authenticatedStore.requireUser();
    expect(user.id).toBe("usr_active");
    expect(user.username).toBe("bob");
    expect(user.role).toBe("admin");
  });

  it("clears error state cleanly with clearError", async () => {
    vi.spyOn(authApi, "login").mockRejectedValue(new Error("Network failure"));

    const store = new AuthStore();
    await expect(store.login({ username: "alex", password: "pwd" })).rejects.toThrow(
      "Network failure",
    );
    expect(store.state.status).toBe("error");
    expect(store.error).toBe("Network failure");

    store.clearError();
    expect(store.state.status).toBe("success");
    expect(store.error).toBeNull();
    expect(() => store.requireUser()).toThrow();
  });

  it("handles session expiration cleanly, transitions to unauthenticated, and toasts", async () => {
    const store = new AuthStore();
    const mockUser: UserDto = {
      id: UserId("usr_active"),
      username: "alice",
      role: "admin",
      createdAt: 100,
    };
    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);
    await store.login({ username: "alice", password: "pwd" });
    expect(store.isAuthenticated).toBe(true);

    store.handleSessionExpired("Custom expiration message");

    expect(store.isAuthenticated).toBe(false);
    expect(() => store.requireUser()).toThrow();
    expect(store.state.status).toBe("success");

    // Idempotent: repeated call when already unauthenticated does not throw or re-toast
    store.handleSessionExpired("Another call");
    expect(store.isAuthenticated).toBe(false);

    store.dispose();
  });
});
