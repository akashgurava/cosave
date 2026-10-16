/**
 * Theme management store for dark mode, light mode, and system preference tracking.
 *
 * Persists the user selection to localStorage and synchronizes the `data-theme`
 * attribute and `dark` class on the root HTML document element.
 */

import {
  expectDocumentElement,
  expectWindow,
  hasLocalStorage,
  hasMatchMedia,
  hasWindow,
} from "./env";
import { createEnum } from "./types";

/**
 * User-configurable theme appearance mode.
 */
export const ThemeMode = createEnum("ThemeMode", {
  Dark: "dark",
  Light: "light",
  System: "system",
});
export type ThemeMode = (typeof ThemeMode.values)[number];

/**
 * Effective rendered color theme resolved against OS preferences.
 */
export const ResolvedTheme = createEnum("ResolvedTheme", {
  Dark: "dark",
  Light: "light",
});
export type ResolvedTheme = (typeof ResolvedTheme.values)[number];

export const THEME_STORAGE_KEY = "cosave-theme";

/**
 * Manages color theme state, persistence in localStorage, and OS preference matching.
 */
export class ThemeStore {
  #mode = $state<ThemeMode>(ThemeMode.Dark);
  #systemPrefersDark = $state<boolean>(true);
  #initialized = false;

  #mediaQuery: MediaQueryList | null = null;
  #mediaListener: ((event: MediaQueryListEvent) => void) | null = null;

  #resolvedTheme = $derived<ResolvedTheme>(
    this.#mode === ThemeMode.System
      ? this.#systemPrefersDark
        ? ResolvedTheme.Dark
        : ResolvedTheme.Light
      : this.#mode,
  );

  #isDark = $derived<boolean>(this.#resolvedTheme === ResolvedTheme.Dark);

  public constructor() {
    if (hasWindow()) {
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

    const win = expectWindow("THEME.INIT.EXPECT_WINDOW");

    if (hasLocalStorage()) {
      const stored = localStorage.getItem(THEME_STORAGE_KEY);
      if (stored === ThemeMode.Dark || stored === ThemeMode.Light || stored === ThemeMode.System) {
        this.#mode = stored;
      } else {
        this.#mode = ThemeMode.Dark;
      }
    }

    if (hasMatchMedia()) {
      this.#mediaQuery = win.matchMedia("(prefers-color-scheme: dark)");
      this.#systemPrefersDark = this.#mediaQuery.matches;

      this.#mediaListener = (event: MediaQueryListEvent) => {
        this.#systemPrefersDark = event.matches;
        this.applyTheme();
      };

      this.#mediaQuery.addEventListener("change", this.#mediaListener);
    }

    this.applyTheme();
    this.#initialized = true;
  }

  /**
   * Updates the selected mode and persists to localStorage.
   */
  public setTheme(mode: ThemeMode): void {
    this.#mode = mode;
    if (hasLocalStorage()) {
      localStorage.setItem(THEME_STORAGE_KEY, mode);
    }
    this.applyTheme();
  }

  /**
   * Toggles between dark and light themes (skipping system).
   */
  public toggleTheme(): void {
    this.setTheme(this.#resolvedTheme === ResolvedTheme.Dark ? ThemeMode.Light : ThemeMode.Dark);
  }

  /**
   * Resets theme back to authoritative default (dark).
   */
  public reset(): void {
    this.setTheme(ThemeMode.Dark);
  }

  /**
   * Unbinds OS theme preference listener.
   */
  public destroy(): void {
    if (this.#mediaQuery !== null && this.#mediaListener !== null) {
      this.#mediaQuery.removeEventListener("change", this.#mediaListener);
      this.#mediaQuery = null;
      this.#mediaListener = null;
    }
    this.#initialized = false;
  }

  /**
   * Sets the `data-theme` attribute and `dark` class on the root HTML element.
   */
  public applyTheme(): void {
    const root = expectDocumentElement("THEME.APPLY_THEME.EXPECT_DOCUMENT_ELEMENT");
    root.setAttribute("data-theme", this.#resolvedTheme);
    if (this.#resolvedTheme === ResolvedTheme.Dark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }
}

export const themeStore = new ThemeStore();
