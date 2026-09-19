import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  api,
  ApiError,
  Code,
  ContractViolationError,
  MemoryTransportAdapter,
  Status,
} from "$lib/api";
import { authApi } from "./api";
import { AuthStore } from "./store";

describe("Auth API & Store Integration (Contract Seam & Envelope Decoders)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  describe("authApi.register", () => {
    it("registers user successfully and decodes UserDto", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", (req) => {
        expect(req.headers["Content-Type"]).toBe("application/json");
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("alice");
        expect(body.password).toBe("secret123");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "usr-alice",
            name: "alice",
            role: "admin",
            created_at: 1700000000,
          },
        };
      });

      const user = await authApi.register({ name: "alice", password: "secret123" });
      expect(user).not.toBeNull();
      expect(user?.id).toBe("usr-alice");
      expect(user?.name).toBe("alice");
      expect(user?.role).toBe("admin");
      expect(user?.created_at).toBe(1700000000);
    });

    it("throws ApiError with isConflict when user already exists", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", () => ({
        code: 409,
        status: "USER_EXISTS",
        data: null,
      }));

      let error: ApiError | null = null;
      try {
        await authApi.register({ name: "alice", password: "pwd" });
      } catch (err) {
        error = err as ApiError;
      }

      expect(error).toBeInstanceOf(ApiError);
      expect(error?.code).toBe(409);
      expect(error?.apiStatus).toBe("USER_EXISTS");
      expect(error?.isConflict).toBe(true);
    });

    it("throws ContractViolationError when response violates UserDto schema", async () => {
      memoryTransport.on("POST", "/api/v1/auth/register", () => ({
        code: 0,
        status: "OK",
        data: { id: "usr-1", name: "alice", role: "superadmin", created_at: "not-a-number" },
      }));

      await expect(authApi.register({ name: "alice", password: "pwd" })).rejects.toThrow(
        ContractViolationError,
      );
    });
  });

  describe("authApi.login", () => {
    it("logs in successfully and returns parsed UserDto", async () => {
      memoryTransport.on("POST", "/api/v1/auth/login", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("bob");
        expect(body.password).toBe("pwd123");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "usr-bob",
            name: "bob",
            role: "member",
            created_at: 1700000500,
          },
        };
      });

      const user = await authApi.login({ name: "bob", password: "pwd123" });
      expect(user).not.toBeNull();
      expect(user?.name).toBe("bob");
      expect(user?.role).toBe("member");
    });

    it("throws ApiError with isUnauthorized on invalid credentials", async () => {
      memoryTransport.on("POST", "/api/v1/auth/login", () => ({
        code: 401,
        status: "INVALID_CREDENTIALS",
        data: null,
      }));

      let error: ApiError | null = null;
      try {
        await authApi.login({ name: "bob", password: "wrong" });
      } catch (err) {
        error = err as ApiError;
      }

      expect(error).toBeInstanceOf(ApiError);
      expect(error?.httpStatus).toBe(401);
      expect(error?.apiStatus).toBe("INVALID_CREDENTIALS");
      expect(error?.isUnauthorized).toBe(true);
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
          name: "charlie",
          role: "member",
          created_at: 1700001000,
        },
      }));

      const me = await authApi.me();
      expect(me.id).toBe("usr-me");
      expect(me.name).toBe("charlie");
      expect(me.role).toBe("member");
    });

    it("throws 401 ApiError with isUnauthorized when session is missing", async () => {
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
      expect(error?.httpStatus).toBe(401);
      expect(error?.apiStatus).toBe("UNAUTHENTICATED");
      expect(error?.isUnauthorized).toBe(true);
    });
  });

  describe("AuthStore Integration with Real API Transport", () => {
    it("initializes session cleanly when authenticated", async () => {
      memoryTransport.on("GET", "/api/v1/auth/me", () => ({
        code: 0,
        status: "OK",
        data: {
          id: "usr-active",
          name: "dana",
          role: "admin",
          created_at: 1700002000,
        },
      }));

      const store = new AuthStore();
      expect(store.isLoading).toBe(true);
      expect(store.isAuthenticated).toBe(false);

      await store.init();

      expect(store.isLoading).toBe(false);
      expect(store.isAuthenticated).toBe(true);
      expect(store.currentUser?.name).toBe("dana");
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
          name: "evan",
          role: "member",
          created_at: 1700003000,
        },
      }));

      memoryTransport.on("POST", "/api/v1/auth/logout", () => ({
        code: 0,
        status: "OK",
        data: null,
      }));

      const store = new AuthStore();
      await store.login({ name: "evan", password: "pwd" });

      expect(store.isAuthenticated).toBe(true);
      expect(store.currentUser?.name).toBe("evan");
      expect(store.error).toBeNull();

      await store.logout();
      expect(store.isAuthenticated).toBe(false);
      expect(store.currentUser).toBeNull();
    });
  });
});
