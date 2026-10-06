/**
 * Reactive authentication store managing session lifecycle, user state, and credentials.
 *
 * Encapsulates user session presence, administrative role status, and error states
 * behind read-only getters and explicit action methods.
 */

import { ApiError, type ErrorPayload } from "$lib/api";
import type { AsyncState } from "$lib/types";
import { authApi } from "./api";
import type { LoginPayload, RegisterPayload, UserDto } from "./types";

function toErrorPayload(err: unknown, fallbackAction: string): ErrorPayload {
  if (err instanceof ApiError) {
    return {
      action: err.action ?? fallbackAction,
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
  #state = $state<AsyncState<UserDto | null>>({ status: "loading" });
  #initPromise: Promise<void> | null = null;

  public constructor() {
    if (typeof window !== "undefined") {
      void this.init();
    }
  }

  public get state(): AsyncState<UserDto | null> {
    return this.#state;
  }

  public get currentUser(): UserDto | null {
    return this.#state.status === "success" ? this.#state.data : null;
  }

  public get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  public get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  public get errorPayload(): ErrorPayload | null {
    return this.#state.status === "error" ? this.#state.error : null;
  }

  public get isAuthenticated(): boolean {
    return this.currentUser !== null;
  }

  public get isAdmin(): boolean {
    return this.currentUser?.role === "admin";
  }

  /**
   * Restores session on startup by checking `/api/v1/auth/me`.
   * Deduplicates concurrent initialization calls.
   */
  public async init(force = false): Promise<void> {
    if (this.#initPromise !== null && !force) {
      return this.#initPromise;
    }
    this.#initPromise = this.#performInit();
    return this.#initPromise;
  }

  async #performInit(): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const user = await authApi.me();
      this.#state = { status: "success", data: user };
      console.info(`[cosave:auth] Active session verified: ${user.username} (${user.role})`);
    } catch {
      this.#state = { status: "success", data: null };
      console.info("[cosave:auth] No active session found (guest)");
    }
  }

  /**
   * Authenticates user with username and password.
   */
  public async login(payload: LoginPayload): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const user = await authApi.login(payload);
      this.#state = { status: "success", data: user };
      this.#initPromise = null;
      console.info(`[cosave:auth] Login successful: ${user?.username ?? payload.username}`);
    } catch (err: unknown) {
      this.#state = { status: "error", error: toErrorPayload(err, "AUTH.LOGIN.FAILED") };
      console.error("[cosave:auth] Login failed:", err);
      throw err;
    }
  }

  /**
   * Registers a new account.
   */
  public async register(payload: RegisterPayload): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const user = await authApi.register(payload);
      this.#state = { status: "success", data: user };
      this.#initPromise = null;
      console.info(`[cosave:auth] Registration successful: ${user?.username ?? payload.username}`);
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
      await authApi.logout();
      console.info("[cosave:auth] User logged out successfully");
    } finally {
      this.#state = { status: "success", data: null };
      this.#initPromise = null;
    }
  }

  /**
   * Clears any active error state back to idle guest state.
   */
  public clearError(): void {
    if (this.#state.status === "error") {
      this.#state = { status: "success", data: null };
    }
  }
}

export const authStore = new AuthStore();
