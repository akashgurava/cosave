import { authApi } from "./api";
import type { LoginPayload, RegisterPayload, UserDto } from "./types";

/**
 * Reactive store managing current session, authentication state, and actions.
 */
export class AuthStore {
  public currentUser = $state<UserDto | null>(null);
  public isLoading = $state<boolean>(true);
  public error = $state<string | null>(null);
  private initialized = false;

  public isAuthenticated = $derived<boolean>(this.currentUser !== null);

  public constructor() {
    if (typeof window !== "undefined") {
      this.init();
    }
  }

  /**
   * Restores session on startup by checking `/api/v1/auth/me`.
   */
  public async init(): Promise<void> {
    if (this.initialized) {
      return;
    }
    this.initialized = true;
    this.isLoading = true;
    try {
      const user = await authApi.me();
      this.currentUser = user;
      this.error = null;
      console.info(`[cosave:auth] Active session verified: ${user.name} (${user.role})`);
    } catch {
      this.currentUser = null;
      console.info("[cosave:auth] No active session found (guest)");
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * Authenticates user with name and password.
   */
  public async login(payload: LoginPayload): Promise<void> {
    this.error = null;
    try {
      const user = await authApi.login(payload);
      this.currentUser = user;
      console.info(`[cosave:auth] Login successful: ${user?.name ?? payload.name}`);
    } catch (err: unknown) {
      if (err instanceof Error) {
        this.error = err.message;
      } else {
        this.error = "Login failed";
      }
      console.error("[cosave:auth] Login failed:", err);
      throw err;
    }
  }

  /**
   * Registers a new account.
   */
  public async register(payload: RegisterPayload): Promise<void> {
    this.error = null;
    try {
      const user = await authApi.register(payload);
      this.currentUser = user;
      console.info(`[cosave:auth] Registration successful: ${user?.name ?? payload.name}`);
    } catch (err: unknown) {
      if (err instanceof Error) {
        this.error = err.message;
      } else {
        this.error = "Registration failed";
      }
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
      this.currentUser = null;
      this.error = null;
    }
  }
}

export const authStore = new AuthStore();
