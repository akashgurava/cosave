/**
 * Authentication and session lifecycle store.
 *
 * Manages user credentials, active session status, and sign out
 * through explicit action methods and the AuthState state machine.
 */

import { ApiError, onUnauthorized, type ErrorPayload } from "$lib/api";
import { toast } from "$lib/toast";
import { expectPresent } from "$lib/types";

import { authApi } from "./api";
import {
  type AuthState,
  type AuthTransport,
  type LoginPayload,
  type RegisterPayload,
  type UserDto,
} from "./types";

function toErrorPayload(err: unknown, fallbackAction: string): ErrorPayload {
  if (err instanceof ApiError) {
    return {
      action: err.action !== null ? err.action : fallbackAction,
      message: err.message,
    };
  }
  if (err instanceof Error) {
    return {
      action: fallbackAction,
      message: err.message,
    };
  }
  return {
    action: fallbackAction,
    message: String(err),
  };
}

/**
 * Reactive store managing current session, authentication state, and actions.
 */
export class AuthStore {
  #state = $state<AuthState>({ status: "init" });
  #transport: AuthTransport;
  #initPromise: Promise<void> | null = null;
  #unsubscribeUnauthorized: (() => void) | null = null;

  public constructor(transport: AuthTransport = authApi) {
    this.#transport = transport;
    this.#unsubscribeUnauthorized = onUnauthorized((err, path) => {
      if (this.isAuthenticated === true && path !== "/api/v1/auth/login") {
        this.handleSessionExpired();
      }
    });

    if (typeof window !== "undefined") {
      void this.init();
    }
  }

  /**
   * True strictly during initial cold boot before session check resolves.
   */
  public get isInit(): boolean {
    return this.#state.status === "init";
  }

  /**
   * True strictly when an active, verified user session exists.
   */
  public get isAuthenticated(): boolean {
    return this.#state.status === "authenticated";
  }

  /**
   * Authoritative non-nullable user accessor. Fails fast if called on unauthenticated routes.
   */
  public requireUser(): UserDto {
    const user = this.#state.status === "authenticated" ? this.#state.user : null;
    return expectPresent(
      user,
      "AUTH.REQUIRE_USER",
      "User session required. Ensure user is authenticated before accessing current user.",
    );
  }

  /**
   * Restores session on startup by checking `/api/v1/auth/me`.
   * Deduplicates concurrent initialization calls.
   */
  public async init(force = false): Promise<void> {
    if (this.#initPromise !== null && force === false) {
      return this.#initPromise;
    }
    this.#initPromise = this.#performInit();
    return this.#initPromise;
  }

  async #performInit(): Promise<void> {
    this.#state = { status: "init" };
    try {
      const user = await this.#transport.me();
      this.#state = { status: "authenticated", user };
      console.info(`[cosave:auth] Active session verified: ${user.username} (${user.role})`);
    } catch {
      this.#state = { status: "unauthenticated" };
      console.info("[cosave:auth] No active session found (guest)");
    }
  }

  /**
   * Authenticates user with username and password.
   */
  public async login(payload: LoginPayload): Promise<UserDto> {
    this.#state = { status: "authenticating" };
    try {
      const user = await this.#transport.login(payload);
      this.#state = { status: "authenticated", user };
      this.#initPromise = null;
      console.info(`[cosave:auth] Login successful: ${user.username}`);
      return user;
    } catch (err: unknown) {
      this.#state = { status: "error", error: toErrorPayload(err, "AUTH.LOGIN.FAILED") };
      console.error("[cosave:auth] Login failed:", err);
      throw err;
    }
  }

  /**
   * Registers a new account.
   */
  public async register(payload: RegisterPayload): Promise<UserDto> {
    this.#state = { status: "authenticating" };
    try {
      const user = await this.#transport.register(payload);
      this.#state = { status: "authenticated", user };
      this.#initPromise = null;
      console.info(`[cosave:auth] Registration successful: ${user.username}`);
      return user;
    } catch (err: unknown) {
      this.#state = { status: "error", error: toErrorPayload(err, "AUTH.REGISTER.FAILED") };
      console.error("[cosave:auth] Registration failed:", err);
      throw err;
    }
  }

  /**
   * Revokes the current session and clears local user state.
   */
  public async logout(): Promise<void> {
    try {
      await this.#transport.logout();
      console.info("[cosave:auth] User logged out successfully");
    } finally {
      this.#state = { status: "unauthenticated" };
      this.#initPromise = null;
    }
  }

  /**
   * Resets active session immediately when an in-flight request returns 401 Unauthorized,
   * liquidating ghost data and displaying an informative notification.
   */
  public handleSessionExpired(message = "Your session has expired. Please sign in again."): void {
    if (this.isAuthenticated === false) {
      return;
    }
    this.#state = { status: "unauthenticated" };
    this.#initPromise = null;
    toast.info(message, {
      duration: 6000,
    });
    console.warn("[cosave:auth] Session expired in-flight. User state cleared.");
  }

  /**
   * Unsubscribes from global interceptor listeners (useful for test isolation).
   */
  public dispose(): void {
    if (this.#unsubscribeUnauthorized !== null) {
      this.#unsubscribeUnauthorized();
      this.#unsubscribeUnauthorized = null;
    }
  }
}

export const authStore = new AuthStore();
