import { describe, it, expect, beforeEach, vi } from "vitest";
import { InvariantViolationError } from "./errors";
import { ThemeStore, ThemeMode, ResolvedTheme, THEME_STORAGE_KEY } from "./theme.svelte";

describe("ThemeStore", () => {
  let mockStorage: Record<string, string> = {};

  beforeEach(() => {
    mockStorage = {};
    vi.stubGlobal("localStorage", {
      getItem: (key: string): string | null => {
        const val = mockStorage[key];
        return val !== undefined ? val : null;
      },
      setItem: (key: string, value: string): void => {
        mockStorage[key] = value;
      },
      removeItem: (key: string): void => {
        delete mockStorage[key];
      },
      clear: (): void => {
        mockStorage = {};
      },
    });

    const mockMatchMedia = (query: string) => ({
      matches: query.includes("dark"),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    });

    vi.stubGlobal("window", {
      matchMedia: mockMatchMedia,
    });

    vi.stubGlobal("matchMedia", mockMatchMedia);

    vi.stubGlobal("document", {
      documentElement: {
        setAttribute: vi.fn(),
        getAttribute: vi.fn(),
        classList: {
          add: vi.fn(),
          remove: vi.fn(),
          contains: vi.fn(),
        },
      },
    });
  });

  it("initializes with default dark theme when no storage exists", () => {
    const store = new ThemeStore();
    expect(store.mode).toBe(ThemeMode.Dark);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);
    expect(store.isDark).toBe(true);
  });

  it("initializes with stored theme if available", () => {
    mockStorage[THEME_STORAGE_KEY] = ThemeMode.Light;
    const store = new ThemeStore();
    expect(store.mode).toBe(ThemeMode.Light);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Light);
    expect(store.isDark).toBe(false);
  });

  it("resolves system theme based on matchMedia", () => {
    mockStorage[THEME_STORAGE_KEY] = ThemeMode.System;
    const store = new ThemeStore();
    expect(store.mode).toBe(ThemeMode.System);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);
  });

  it("updates theme and storage with setTheme", () => {
    const store = new ThemeStore();
    store.setTheme(ThemeMode.Dark);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);

    store.setTheme(ThemeMode.Light);
    expect(store.mode).toBe(ThemeMode.Light);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Light);
    expect(mockStorage[THEME_STORAGE_KEY]).toBe(ThemeMode.Light);

    store.setTheme(ThemeMode.Dark);
    expect(store.mode).toBe(ThemeMode.Dark);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);
    expect(mockStorage[THEME_STORAGE_KEY]).toBe(ThemeMode.Dark);
  });

  it("toggles between dark and light themes using toggleTheme", () => {
    const store = new ThemeStore();
    store.setTheme(ThemeMode.Dark);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);

    store.toggleTheme();
    expect(store.resolvedTheme).toBe(ResolvedTheme.Light);
    expect(store.mode).toBe(ThemeMode.Light);

    store.toggleTheme();
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);
    expect(store.mode).toBe(ThemeMode.Dark);
  });

  it("resets back to authoritative default with reset", () => {
    const store = new ThemeStore();
    store.setTheme(ThemeMode.Light);
    expect(store.mode).toBe(ThemeMode.Light);

    store.reset();
    expect(store.mode).toBe(ThemeMode.Dark);
    expect(store.resolvedTheme).toBe(ResolvedTheme.Dark);
  });

  it("unbinds listeners on destroy", () => {
    const store = new ThemeStore();
    store.destroy();
    // Subsequent calls or destruction should not throw
    expect(() => store.destroy()).not.toThrow();
  });

  it("throws InvariantViolationError if init is called when window is undefined", () => {
    vi.stubGlobal("window", undefined);
    const store = new ThemeStore();
    try {
      store.init();
      expect.unreachable("should have thrown InvariantViolationError");
    } catch (err) {
      expect(err).toBeInstanceOf(InvariantViolationError);
      expect((err as InvariantViolationError).action).toBe("THEME.INIT.EXPECT_WINDOW");
    }
  });
});
