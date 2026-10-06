import { describe, it, expect, vi, beforeEach } from "vitest";
import { AuthStore } from "./store.svelte";
import { authApi } from "./api";
import type { UserDto } from "./types";

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
      username: "tester",
      role: "admin",
      createdAt: 1700000000,
    };

    vi.spyOn(authApi, "login").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.login({ username: "tester", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.currentUser).toEqual(mockUser);
    expect(store.error).toBeNull();
  });

  it("registers user and updates currentUser", async () => {
    const mockUser: UserDto = {
      id: "user-456",
      username: "newbie",
      role: "member",
      createdAt: 1700000100,
    };

    vi.spyOn(authApi, "register").mockResolvedValue(mockUser);

    const store = new AuthStore();
    await store.register({ username: "newbie", password: "password123" });

    expect(store.isAuthenticated).toBe(true);
    expect(store.currentUser).toEqual(mockUser);
    expect(store.error).toBeNull();
  });

  it("sets error and rethrows when login fails", async () => {
    vi.spyOn(authApi, "login").mockRejectedValue(new Error("Invalid credentials"));

    const store = new AuthStore();
    await expect(store.login({ username: "bad", password: "pwd" })).rejects.toThrow("Invalid credentials");

    expect(store.error).toBe("Invalid credentials");
    expect(store.isAuthenticated).toBe(false);
  });

  it("clears currentUser on logout", async () => {
    const mockUser: UserDto = {
      id: "u1",
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
    expect(store.currentUser).toBeNull();
    expect(store.error).toBeNull();
  });
});
