export type ThemeMode = "dark" | "light" | "system";
export type ResolvedTheme = "dark" | "light";

export const THEME_STORAGE_KEY = "cosave-theme";

/**
 * Manages color theme state, persistence in localStorage, and OS preference matching.
 */
export class ThemeStore {
  public mode = $state<ThemeMode>("dark");
  public systemPrefersDark = $state<boolean>(true);
  private initialized = false;

  /**
   * Resolved theme ("dark" or "light") taking into account system preference if mode is "system".
   */
  public resolvedTheme = $derived<ResolvedTheme>(
    this.mode === "system" ? (this.systemPrefersDark ? "dark" : "light") : this.mode,
  );

  public isDark = $derived<boolean>(this.resolvedTheme === "dark");

  public constructor() {
    if (
      typeof window !== "undefined" ||
      typeof document !== "undefined" ||
      typeof localStorage !== "undefined"
    ) {
      this.init();
    }
  }

  /**
   * Initializes theme from localStorage and sets up system preference listeners.
   */
  public init(): void {
    if (this.initialized) {
      return;
    }

    if (typeof localStorage !== "undefined") {
      const stored = localStorage.getItem(THEME_STORAGE_KEY);
      if (stored === "dark" || stored === "light" || stored === "system") {
        this.mode = stored;
      } else {
        this.mode = "dark";
      }
    }

    if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      this.systemPrefersDark = mediaQuery.matches;

      mediaQuery.addEventListener("change", (event: MediaQueryListEvent) => {
        this.systemPrefersDark = event.matches;
        this.applyTheme();
      });
    }

    this.initialized = true;
    this.applyTheme();
  }

  /**
   * Updates the selected mode and persists to localStorage.
   */
  public setTheme(mode: ThemeMode): void {
    this.mode = mode;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    }
    this.applyTheme();
  }

  /**
   * Toggles between dark and light themes.
   */
  public toggleTheme(): void {
    const next: ResolvedTheme = this.resolvedTheme === "dark" ? "light" : "dark";
    this.setTheme(next);
  }

  /**
   * Sets the `data-theme` attribute and `dark` class on the root HTML element.
   */
  public applyTheme(): void {
    if (typeof document !== "undefined" && document.documentElement) {
      document.documentElement.setAttribute("data-theme", this.resolvedTheme);
      if (document.documentElement.classList) {
        if (this.resolvedTheme === "dark") {
          document.documentElement.classList.add("dark");
        } else {
          document.documentElement.classList.remove("dark");
        }
      }
    }
  }
}

export const themeStore = new ThemeStore();
