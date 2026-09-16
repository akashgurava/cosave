import { authApi, type LoginPayload, type RegisterPayload, type UserDto } from "$lib/api";

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
      const res = await authApi.me();
      this.currentUser = res.data;
      this.error = null;
    } catch {
      this.currentUser = null;
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
      const res = await authApi.login(payload);
      this.currentUser = res.data;
    } catch (err: unknown) {
      if (err instanceof Error) {
        this.error = err.message;
      } else {
        this.error = "Login failed";
      }
      throw err;
    }
  }

  /**
   * Registers a new account.
   */
  public async register(payload: RegisterPayload): Promise<void> {
    this.error = null;
    try {
      const res = await authApi.register(payload);
      this.currentUser = res.data;
    } catch (err: unknown) {
      if (err instanceof Error) {
        this.error = err.message;
      } else {
        this.error = "Registration failed";
      }
      throw err;
    }
  }

  /**
   * Revokes the current session and clears local user state.
   */
  public async logout(): Promise<void> {
    try {
      await authApi.logout();
    } finally {
      this.currentUser = null;
      this.error = null;
    }
  }
}

export const authStore = new AuthStore();
