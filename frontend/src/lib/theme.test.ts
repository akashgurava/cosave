import { describe, it, expect, beforeEach, vi } from "vitest";
import { ThemeStore, THEME_STORAGE_KEY } from "./theme.svelte";

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
    expect(store.mode).toBe("dark");
    expect(store.resolvedTheme).toBe("dark");
    expect(store.isDark).toBe(true);
  });

  it("initializes with stored theme if available", () => {
    mockStorage[THEME_STORAGE_KEY] = "light";
    const store = new ThemeStore();
    expect(store.mode).toBe("light");
    expect(store.resolvedTheme).toBe("light");
    expect(store.isDark).toBe(false);
  });

  it("resolves system theme based on matchMedia", () => {
    mockStorage[THEME_STORAGE_KEY] = "system";
    const store = new ThemeStore();
    expect(store.mode).toBe("system");
    expect(store.resolvedTheme).toBe("dark");
  });

  it("updates theme and storage with setTheme", () => {
    const store = new ThemeStore();
    store.setTheme("dark");
    expect(store.resolvedTheme).toBe("dark");

    store.setTheme("light");
    expect(store.mode).toBe("light");
    expect(store.resolvedTheme).toBe("light");
    expect(mockStorage[THEME_STORAGE_KEY]).toBe("light");

    store.setTheme("dark");
    expect(store.mode).toBe("dark");
    expect(store.resolvedTheme).toBe("dark");
    expect(mockStorage[THEME_STORAGE_KEY]).toBe("dark");
  });

  it("toggles between dark and light themes using toggleTheme", () => {
    const store = new ThemeStore();
    store.setTheme("dark");
    expect(store.resolvedTheme).toBe("dark");

    store.toggleTheme();
    expect(store.resolvedTheme).toBe("light");
    expect(store.mode).toBe("light");

    store.toggleTheme();
    expect(store.resolvedTheme).toBe("dark");
    expect(store.mode).toBe("dark");
  });

  it("resets back to authoritative default with reset", () => {
    const store = new ThemeStore();
    store.setTheme("light");
    expect(store.mode).toBe("light");

    store.reset();
    expect(store.mode).toBe("dark");
    expect(store.resolvedTheme).toBe("dark");
  });

  it("unbinds listeners on destroy", () => {
    const store = new ThemeStore();
    store.destroy();
    // Subsequent calls or destruction should not throw
    expect(() => store.destroy()).not.toThrow();
  });
});
