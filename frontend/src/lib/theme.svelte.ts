/**
 * Theme management store for dark mode, light mode, and system preference tracking.
 *
 * Persists the user selection to localStorage and synchronizes the `data-theme`
 * attribute and `dark` class on the root HTML document element.
 */

export type ThemeMode = "dark" | "light" | "system";
export type ResolvedTheme = "dark" | "light";

export const THEME_STORAGE_KEY = "cosave-theme";

/**
 * Manages color theme state, persistence in localStorage, and OS preference matching.
 */
export class ThemeStore {
  #mode = $state<ThemeMode>("dark");
  #systemPrefersDark = $state<boolean>(true);
  #initialized = false;

  #mediaQuery: MediaQueryList | null = null;
  #mediaListener: ((event: MediaQueryListEvent) => void) | null = null;

  #resolvedTheme = $derived<ResolvedTheme>(
    this.#mode === "system" ? (this.#systemPrefersDark ? "dark" : "light") : this.#mode,
  );

  #isDark = $derived<boolean>(this.#resolvedTheme === "dark");

  public constructor() {
    if (typeof window !== "undefined") {
      this.init();
    }
  }

  public get mode(): ThemeMode {
    return this.#mode;
  }

  public get resolvedTheme(): ResolvedTheme {
    return this.#resolvedTheme;
  }

  public get isDark(): boolean {
    return this.#isDark;
  }

  /**
   * Initializes theme from localStorage and sets up system preference listeners.
   */
  public init(): void {
    if (this.#initialized) {
      return;
    }

    if (typeof localStorage !== "undefined") {
      const stored = localStorage.getItem(THEME_STORAGE_KEY);
      if (stored === "dark" || stored === "light" || stored === "system") {
        this.#mode = stored;
      } else {
        this.#mode = "dark";
      }
    }

    if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
      this.#mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      this.#systemPrefersDark = this.#mediaQuery.matches;

      this.#mediaListener = (event: MediaQueryListEvent) => {
        this.#systemPrefersDark = event.matches;
        this.applyTheme();
      };

      if (typeof this.#mediaQuery.addEventListener === "function") {
        this.#mediaQuery.addEventListener("change", this.#mediaListener);
      } else if (
        typeof (this.#mediaQuery as unknown as { addListener: (cb: unknown) => void })
          .addListener === "function"
      ) {
        (this.#mediaQuery as unknown as { addListener: (cb: unknown) => void }).addListener(
          this.#mediaListener,
        );
      }
    }

    this.#initialized = true;
    this.applyTheme();
  }

  /**
   * Updates the selected mode and persists to localStorage.
   */
  public setTheme(mode: ThemeMode): void {
    this.#mode = mode;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    }
    this.applyTheme();
  }

  /**
   * Toggles between dark and light themes (skipping system).
   */
  public toggleTheme(): void {
    this.setTheme(this.#resolvedTheme === "dark" ? "light" : "dark");
  }

  /**
   * Resets theme back to authoritative default (dark).
   */
  public reset(): void {
    this.setTheme("dark");
  }

  /**
   * Unbinds OS theme preference listener.
   */
  public destroy(): void {
    if (this.#mediaQuery !== null && this.#mediaListener !== null) {
      if (typeof this.#mediaQuery.removeEventListener === "function") {
        this.#mediaQuery.removeEventListener("change", this.#mediaListener);
      } else if (
        typeof (this.#mediaQuery as unknown as { removeListener: (cb: unknown) => void })
          .removeListener === "function"
      ) {
        (this.#mediaQuery as unknown as { removeListener: (cb: unknown) => void }).removeListener(
          this.#mediaListener,
        );
      }
      this.#mediaQuery = null;
      this.#mediaListener = null;
    }
    this.#initialized = false;
  }

  /**
   * Sets the `data-theme` attribute and `dark` class on the root HTML element.
   */
  public applyTheme(): void {
    if (typeof document !== "undefined" && document.documentElement) {
      document.documentElement.setAttribute("data-theme", this.#resolvedTheme);
      if (document.documentElement.classList) {
        if (this.#resolvedTheme === "dark") {
          document.documentElement.classList.add("dark");
        } else {
          document.documentElement.classList.remove("dark");
        }
      }
    }
  }
}

export const themeStore = new ThemeStore();
