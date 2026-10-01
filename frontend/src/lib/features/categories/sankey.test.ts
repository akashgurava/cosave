import { describe, expect, it } from "vitest";
import { getTypeColor, hexToRgba, isColorUsed, projectSankeyGraph } from "./sankey";
import type { PresentationCategoryItem, TransactionTypeItem } from "./types";

describe("sankey projection module", () => {
  const mockTypes: TransactionTypeItem[] = [
    { id: 1, name: "Income", color: "#10b981", color_id: 1, categories: [] },
    { id: 2, name: "Expense", color: "#f43f5e", color_id: 2, categories: [] },
  ];

  const mockCategories: PresentationCategoryItem[] = [
    {
      id: 10,
      name: "Salary",
      type: "Income",
      typeId: 1,
      subcategories: [{ id: 100, name: "Tech Job" }],
    },
    {
      id: 20,
      name: "Housing",
      type: "Expense",
      typeId: 2,
      subcategories: [],
    },
  ];

  it("converts hex to rgba correctly", () => {
    expect(hexToRgba("#10b981", 0.5)).toBe("rgba(16, 185, 129, 0.5)");
    expect(hexToRgba("ffffff", 1)).toBe("rgba(255, 255, 255, 1)");
  });

  it("resolves type colors with fallbacks", () => {
    const incomeColor = getTypeColor(mockTypes, "Income");
    expect(incomeColor.solid).toBe("#10b981");
    expect(incomeColor.subtle).toBe("rgba(16, 185, 129, 0.25)");

    const fallbackColor = getTypeColor(mockTypes, "Unknown");
    expect(fallbackColor.solid).toBe("#71717a");
  });

  it("checks whether a color is already in use", () => {
    expect(isColorUsed(mockTypes, "#10b981")).toBe(true);
    expect(isColorUsed(mockTypes, "#10b981", "Income")).toBe(false);
    expect(isColorUsed(mockTypes, "#000000")).toBe(false);
  });

  it("projects nodes and links with multi-level depth", () => {
    const { nodes, links } = projectSankeyGraph(mockTypes, mockCategories, "All");

    // Nodes for types, categories, subcategories
    expect(nodes.some((n) => n.name === "type:Income" && n.depth === 0)).toBe(true);
    expect(nodes.some((n) => n.name === "cat:10" && n.depth === 2)).toBe(true);
    expect(nodes.some((n) => n.name === "sub:10:100" && n.depth === 5)).toBe(true);

    // Deepened nodes carry resolved domain entities directly
    const subNode = nodes.find((n) => n.name === "sub:10:100");
    expect(subNode?.entity).toEqual({
      id: 100,
      type: "Income",
      kind: "subcategory",
      name: "Tech Job",
      parentName: "Salary",
      categoryId: 10,
    });

    // Links between Type -> Category and Category -> Subcategory
    expect(links.some((l) => l.source === "type:Income" && l.target === "cat:10")).toBe(true);
    expect(links.some((l) => l.source === "cat:10" && l.target === "sub:10:100")).toBe(true);
  });

  it("filters projection by active filter", () => {
    const { nodes, links } = projectSankeyGraph(mockTypes, mockCategories, "Income");

    expect(nodes.some((n) => n.name === "type:Income")).toBe(true);
    expect(nodes.some((n) => n.name === "type:Expense")).toBe(false);
    expect(links.every((l) => l.source.includes("Income") || l.source.includes("cat:10"))).toBe(
      true,
    );
  });
});
