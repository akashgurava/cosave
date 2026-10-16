import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  hasWindow,
  expectWindow,
  hasLocalStorage,
  hasMatchMedia,
  hasDocument,
  hasDocumentElement,
  expectDocumentElement,
} from "./env";
import { InvariantViolationError } from "./errors";

describe("env runtime capability helpers", () => {
  beforeEach(() => {
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe("hasWindow", () => {
    it("returns true when window is defined", () => {
      vi.stubGlobal("window", {});
      expect(hasWindow()).toBe(true);
    });

    it("returns false when window is undefined", () => {
      vi.stubGlobal("window", undefined);
      expect(hasWindow()).toBe(false);
    });
  });

  describe("expectWindow", () => {
    it("returns window when defined", () => {
      const mockWindow = { customProp: "test" } as unknown as Window & typeof globalThis;
      vi.stubGlobal("window", mockWindow);
      expect(expectWindow("TEST.ACTION")).toBe(mockWindow);
    });

    it("throws InvariantViolationError when window is undefined", () => {
      vi.stubGlobal("window", undefined);
      expect(() => expectWindow("TEST.ACTION")).toThrow(InvariantViolationError);
      try {
        expectWindow("TEST.ACTION");
      } catch (err) {
        expect((err as InvariantViolationError).action).toBe("TEST.ACTION");
      }
    });
  });

  describe("hasLocalStorage", () => {
    it("returns true when localStorage is defined", () => {
      vi.stubGlobal("localStorage", {
        getItem: vi.fn(),
        setItem: vi.fn(),
      });
      expect(hasLocalStorage()).toBe(true);
    });

    it("returns false when localStorage is undefined", () => {
      vi.stubGlobal("localStorage", undefined);
      expect(hasLocalStorage()).toBe(false);
    });
  });

  describe("hasMatchMedia", () => {
    it("returns true when window and window.matchMedia function are defined", () => {
      vi.stubGlobal("window", {
        matchMedia: vi.fn(),
      });
      expect(hasMatchMedia()).toBe(true);
    });

    it("returns false when matchMedia is not a function", () => {
      vi.stubGlobal("window", {
        matchMedia: "not-a-function",
      });
      expect(hasMatchMedia()).toBe(false);
    });

    it("returns false when matchMedia is undefined on window", () => {
      vi.stubGlobal("window", {});
      expect(hasMatchMedia()).toBe(false);
    });

    it("returns false when window is undefined", () => {
      vi.stubGlobal("window", undefined);
      expect(hasMatchMedia()).toBe(false);
    });
  });

  describe("hasDocument", () => {
    it("returns true when document is defined", () => {
      vi.stubGlobal("document", {});
      expect(hasDocument()).toBe(true);
    });

    it("returns false when document is undefined", () => {
      vi.stubGlobal("document", undefined);
      expect(hasDocument()).toBe(false);
    });
  });

  describe("hasDocumentElement", () => {
    it("returns true when document.documentElement exists", () => {
      const mockElement = {};
      vi.stubGlobal("document", { documentElement: mockElement });
      expect(hasDocumentElement()).toBe(true);
    });

    it("returns false when documentElement is null", () => {
      vi.stubGlobal("document", { documentElement: null });
      expect(hasDocumentElement()).toBe(false);
    });

    it("returns false when documentElement is undefined", () => {
      vi.stubGlobal("document", { documentElement: undefined });
      expect(hasDocumentElement()).toBe(false);
    });

    it("returns false when document is undefined", () => {
      vi.stubGlobal("document", undefined);
      expect(hasDocumentElement()).toBe(false);
    });
  });

  describe("expectDocumentElement", () => {
    it("returns document.documentElement when present and non-null", () => {
      const mockElement = {} as HTMLElement;
      vi.stubGlobal("document", { documentElement: mockElement });
      expect(expectDocumentElement("TEST.ACTION")).toBe(mockElement);
    });

    it("throws InvariantViolationError when documentElement is null", () => {
      vi.stubGlobal("document", { documentElement: null });
      expect(() => expectDocumentElement("TEST.ACTION")).toThrow(InvariantViolationError);
      try {
        expectDocumentElement("TEST.ACTION");
      } catch (err) {
        expect((err as InvariantViolationError).action).toBe("TEST.ACTION");
      }
    });

    it("throws InvariantViolationError when documentElement is undefined", () => {
      vi.stubGlobal("document", { documentElement: undefined });
      expect(() => expectDocumentElement("TEST.ACTION")).toThrow(InvariantViolationError);
    });

    it("throws InvariantViolationError when document is undefined", () => {
      vi.stubGlobal("document", undefined);
      expect(() => expectDocumentElement("TEST.ACTION")).toThrow(InvariantViolationError);
    });
  });
});
