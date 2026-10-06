import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, Code, ContractViolationError, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { authApi } from "./api";
import { AuthStore } from "./store.svelte";

describe("Auth API & Store Integration (Contract Seam & Envelope Decoders)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: (() => void) | undefined;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    if (restoreTransport !== undefined) {
      restoreTransport();
    }
  });

  describe("authApi.register", () => {
    it("registers user successfully and decodes UserDto", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", (req) => {
        expect(req.headers["Content-Type"]).toBe("application/json");
        const body = JSON.parse(req.body ?? "{}");
        expect(body.username).toBe("alice");
        expect(body.password).toBe("secret123");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "usr-alice",
            username: "alice",
            role: "admin",
            createdAt: 1700000000,
          },
        };
      });

      const user = await authApi.register({ username: "alice", password: "secret123" });
      expect(user.id).toBe("usr-alice");
      expect(user.username).toBe("alice");
      expect(user.role).toBe("admin");
      expect(user.createdAt).toBe(1700000000);
    });

    it("throws ApiError when user already exists", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", () => ({
        code: 409,
        status: "USER_ALREADY_EXISTS",
        data: null,
      }));

      let error: ApiError | null = null;
      try {
        await authApi.register({ username: "alice", password: "pwd" });
      } catch (err) {
        error = err as ApiError;
      }

      expect(error).toBeInstanceOf(ApiError);
      expect(error).not.toBeNull();
      if (error !== null) {
        expect(error.code).toBe(409);
        expect(error.apiStatus).toBe("USER_ALREADY_EXISTS");
      }
    });

    it("throws ContractViolationError when response violates UserDto schema", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", () => ({
        code: 0,
        status: "OK",
        data: { id: "usr-1", username: "alice", role: "superadmin", createdAt: "not-a-number" },
      }));

      await expect(authApi.register({ username: "alice", password: "pwd" })).rejects.toThrow(
        ContractViolationError,
      );
    });
  });

  describe("authApi.login", () => {
    it("logs in successfully and returns parsed UserDto", async () => {
      memoryTransport.on("POST", "/api/v1/auth/login", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.username).toBe("bob");
        expect(body.password).toBe("pwd123");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "usr-bob",
            username: "bob",
            role: "member",
            createdAt: 1700000500,
          },
        };
      });

      const user = await authApi.login({ username: "bob", password: "pwd123" });
      expect(user.username).toBe("bob");
      expect(user.role).toBe("member");
    });

    it("throws ApiError on invalid credentials", async () => {
      memoryTransport.on("POST", "/api/v1/auth/login", () => ({
        code: 401,
        status: "INVALID_CREDENTIALS",
        data: null,
      }));

      let error: ApiError | null = null;
      try {
        await authApi.login({ username: "bob", password: "wrong" });
      } catch (err) {
        error = err as ApiError;
      }

      expect(error).toBeInstanceOf(ApiError);
      expect(error).not.toBeNull();
      if (error !== null) {
        expect(error.httpStatus).toBe(401);
        expect(error.apiStatus).toBe("INVALID_CREDENTIALS");
      }
    });
  });

  describe("authApi.logout", () => {
    it("posts to logout and validates null response", async () => {
      let logoutCalled = false;
      memoryTransport.on("POST", "/api/v1/auth/logout", () => {
        logoutCalled = true;
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const res = await authApi.logout();
      expect(logoutCalled).toBe(true);
      expect(res).toBeNull();
    });
  });

  describe("authApi.me", () => {
    it("fetches active user profile successfully", async () => {
      memoryTransport.on("GET", "/api/v1/auth/me", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          id: "usr-me",
          username: "charlie",
          role: "member",
          createdAt: 1700001000,
        },
      }));

      const me = await authApi.me();
      expect(me.id).toBe("usr-me");
      expect(me.username).toBe("charlie");
      expect(me.role).toBe("member");
    });

    it("throws 401 ApiError when session is missing", async () => {
      memoryTransport.on("GET", "/api/v1/auth/me", () => ({
        code: 401,
        status: "UNAUTHENTICATED",
        data: null,
      }));

      let error: ApiError | null = null;
      try {
        await authApi.me();
      } catch (err) {
        error = err as ApiError;
      }

      expect(error).toBeInstanceOf(ApiError);
      expect(error).not.toBeNull();
      if (error !== null) {
        expect(error.httpStatus).toBe(401);
        expect(error.apiStatus).toBe("UNAUTHENTICATED");
      }
    });
  });

  describe("AuthStore Integration with Real API Transport", () => {
    it("initializes session cleanly when authenticated", async () => {
      memoryTransport.on("GET", "/api/v1/auth/me", () => ({
        code: 0,
        status: "OK",
        data: {
          id: "usr-active",
          username: "dana",
          role: "admin",
          createdAt: 1700002000,
        },
      }));

      const store = new AuthStore();
      expect(store.isLoading).toBe(true);
      expect(store.isAuthenticated).toBe(false);

      await store.init();

      expect(store.isLoading).toBe(false);
      expect(store.isAuthenticated).toBe(true);
      expect(store.currentUser).not.toBeNull();
      if (store.currentUser !== null) {
        expect(store.currentUser.username).toBe("dana");
      }
    });

    it("initializes to guest state without throwing when unauthenticated (401)", async () => {
      memoryTransport.on("GET", "/api/v1/auth/me", () => ({
        code: 401,
        status: "UNAUTHENTICATED",
        data: null,
      }));

      const store = new AuthStore();
      await store.init();

      expect(store.isLoading).toBe(false);
      expect(store.isAuthenticated).toBe(false);
      expect(store.currentUser).toBeNull();
    });

    it("updates store state on successful login and clears on logout", async () => {
      memoryTransport.on("POST", "/api/v1/auth/login", () => ({
        code: 0,
        status: "OK",
        data: {
          id: "usr-session",
          username: "evan",
          role: "member",
          createdAt: 1700003000,
        },
      }));

      memoryTransport.on("POST", "/api/v1/auth/logout", () => ({
        code: 0,
        status: "OK",
        data: null,
      }));

      const store = new AuthStore();
      await store.login({ username: "evan", password: "pwd" });

      expect(store.isAuthenticated).toBe(true);
      expect(store.currentUser).not.toBeNull();
      if (store.currentUser !== null) {
        expect(store.currentUser.username).toBe("evan");
      }
      expect(store.error).toBeNull();

      await store.logout();
      expect(store.isAuthenticated).toBe(false);
      expect(store.currentUser).toBeNull();
    });
  });
});
