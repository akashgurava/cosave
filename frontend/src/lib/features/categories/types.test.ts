import { describe, it, expect } from "vitest";
import {
  parseColorOption,
  parseSubcategoryItem,
  parseCategoryItem,
  parseTransactionTypeItem,
  parseCategoryHierarchyResponse,
} from "./types";
import { ContractViolationError } from "$lib/api/contracts";

describe("Categories Types & Schema Decoders", () => {
  describe("parseColorOption", () => {
    it("parses valid color option", () => {
      const color = parseColorOption({ id: 1, name: "Emerald", hex: "#10b981" });
      expect(color.id).toBe(1);
      expect(color.name).toBe("Emerald");
      expect(color.hex).toBe("#10b981");
      expect(Object.isFrozen(color)).toBe(true);
    });

    it("throws ContractViolationError on invalid fields", () => {
      expect(() => parseColorOption(null)).toThrow(ContractViolationError);
      expect(() => parseColorOption({ id: "1", name: "Emerald", hex: "#10b981" })).toThrow(
        ContractViolationError,
      );
      expect(() => parseColorOption({ id: 1, name: 123, hex: "#10b981" })).toThrow(
        ContractViolationError,
      );
      expect(() => parseColorOption({ id: 1, name: "Emerald", hex: 123 })).toThrow(
        ContractViolationError,
      );
    });
  });

  describe("parseSubcategoryItem", () => {
    it("parses valid subcategory item", () => {
      const sub = parseSubcategoryItem({ id: 10, name: "Rent" });
      expect(sub.id).toBe(10);
      expect(sub.name).toBe("Rent");
      expect(Object.isFrozen(sub)).toBe(true);
    });

    it("throws ContractViolationError on invalid fields", () => {
      expect(() => parseSubcategoryItem(null)).toThrow(ContractViolationError);
      expect(() => parseSubcategoryItem({ id: "10", name: "Rent" })).toThrow(
        ContractViolationError,
      );
      expect(() => parseSubcategoryItem({ id: 10, name: null })).toThrow(ContractViolationError);
    });
  });

  describe("parseCategoryItem", () => {
    it("parses valid category item with subcategories", () => {
      const cat = parseCategoryItem({
        id: 20,
        name: "Housing",
        subcategories: [{ id: 10, name: "Rent" }],
      });
      expect(cat.id).toBe(20);
      expect(cat.name).toBe("Housing");
      expect(cat.subcategories).toHaveLength(1);
      expect(cat.subcategories[0]?.name).toBe("Rent");
    });

    it("throws ContractViolationError when subcategories is not array", () => {
      expect(() => parseCategoryItem({ id: 20, name: "Housing", subcategories: null })).toThrow(
        ContractViolationError,
      );
    });
  });

  describe("parseTransactionTypeItem", () => {
    it("parses valid type item", () => {
      const type = parseTransactionTypeItem({
        id: 1,
        name: "Expense",
        color: "#f43f5e",
        colorId: 2,
        categories: [],
      });
      expect(type.id).toBe(1);
      expect(type.name).toBe("Expense");
      expect(type.color).toBe("#f43f5e");
      expect(type.colorId).toBe(2);
      expect(type.categories).toEqual([]);
    });

    it("throws ContractViolationError on missing required fields", () => {
      expect(() => parseTransactionTypeItem({ id: 1, name: "Expense" })).toThrow(
        ContractViolationError,
      );
    });
  });

  describe("parseCategoryHierarchyResponse", () => {
    it("parses hierarchy response", () => {
      const hierarchy = parseCategoryHierarchyResponse({
        types: [
          {
            id: 1,
            name: "Income",
            color: "#10b981",
            colorId: 1,
            categories: [],
          },
        ],
        colors: [{ id: 1, name: "Emerald", hex: "#10b981" }],
      });
      expect(hierarchy.types).toHaveLength(1);
      expect(hierarchy.colors).toHaveLength(1);
      expect(hierarchy.types[0]?.name).toBe("Income");
    });

    it("throws ContractViolationError when types is missing", () => {
      expect(() => parseCategoryHierarchyResponse({})).toThrow(ContractViolationError);
    });
  });
});
