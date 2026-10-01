import { describe, it, expect, vi, beforeEach } from "vitest";
import { CategoryStore } from "./store";
import { categoriesApi } from "./api";
import {
  PRESET_COLORS,
  type CategoryHierarchyResponse,
  type CategoryItem,
  type SubcategoryItem,
  type TransactionTypeItem,
} from "./types";

const mockDefaults: CategoryHierarchyResponse = {
  types: [
    {
      id: 1,
      name: "Income",
      color: "#10b981",
      color_id: 1,
      categories: [
        {
          id: 10,
          name: "Salary",
          subcategories: [
            { id: 100, name: "Primary Employer" },
            { id: 101, name: "Bonus" },
          ],
        },
        {
          id: 11,
          name: "Freelance",
          subcategories: [
            { id: 102, name: "Consulting" },
            { id: 103, name: "Retainers" },
          ],
        },
      ],
    },
    {
      id: 2,
      name: "Expense",
      color: "#f43f5e",
      color_id: 2,
      categories: [
        {
          id: 20,
          name: "Housing",
          subcategories: [
            { id: 200, name: "Rent" },
            { id: 201, name: "Utilities" },
          ],
        },
        {
          id: 21,
          name: "Food",
          subcategories: [
            { id: 202, name: "Groceries" },
            { id: 203, name: "Dining Out" },
          ],
        },
        {
          id: 22,
          name: "Transport",
          subcategories: [
            { id: 204, name: "Fuel" },
            { id: 205, name: "Public Transit" },
          ],
        },
        {
          id: 23,
          name: "Personal",
          subcategories: [{ id: 206, name: "Gym & Fitness" }],
        },
      ],
    },
    {
      id: 3,
      name: "Transfer",
      color: "#71717a",
      color_id: 3,
      categories: [
        {
          id: 30,
          name: "Internal",
          subcategories: [
            { id: 300, name: "Checking to Savings" },
            { id: 301, name: "Emergency Fund" },
          ],
        },
      ],
    },
    {
      id: 4,
      name: "Invest",
      color: "#3b82f6",
      color_id: 4,
      categories: [
        {
          id: 40,
          name: "Equities",
          subcategories: [{ id: 400, name: "Index ETFs" }],
        },
      ],
    },
  ],
  colors: [...PRESET_COLORS],
};

describe("CategoryStore (Frontend Mirror of Backend SSOT)", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("starts empty without hardcoded initial data and populates from backend load()", async () => {
    const store = new CategoryStore();
    expect(store.types).toHaveLength(0);
    expect(store.categories).toHaveLength(0);
    expect(store.isLoaded).toBe(false);

    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(mockDefaults);

    await store.load();

    expect(store.isLoaded).toBe(true);
    expect(store.types).toHaveLength(4);
    expect(store.categories).toHaveLength(8);

    const totalSubs = store.categories.reduce((acc, c) => acc + c.subcategories.length, 0);
    expect(totalSubs).toBe(14);
  });

  it("delegates addType to categoriesApi.createType", async () => {
    const store = new CategoryStore();
    const createdType: TransactionTypeItem = {
      id: 5,
      name: "Savings",
      color: "#f59e0b",
      color_id: 5,
      categories: [],
    };

    const spy = vi.spyOn(categoriesApi, "createType").mockResolvedValue(createdType);

    const res = await store.addType("Savings", 5);
    expect(spy).toHaveBeenCalledWith({ name: "Savings", color_id: 5 });
    expect(res).toEqual(createdType);
    expect(store.types).toContainEqual(createdType);
  });

  it("delegates updateTypeColor to categoriesApi.updateTypeColor with CQS acknowledgement", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const spy = vi.spyOn(categoriesApi, "updateTypeColor").mockResolvedValue(null);

    const success = await store.updateTypeColor("Income", 5);
    expect(spy).toHaveBeenCalledWith(1, { color_id: 5 });
    expect(success).toBe(true);
    expect(store.getType("Income")?.color_id).toBe(5);
  });

  it("delegates deleteType to categoriesApi.deleteType and cascades local state", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const spy = vi.spyOn(categoriesApi, "deleteType").mockResolvedValue(null);

    const success = await store.deleteType("Income");
    expect(spy).toHaveBeenCalledWith(1);
    expect(success).toBe(true);
    expect(store.getType("Income")).toBeNull();
    expect(store.categories.some((c) => c.type === "Income")).toBe(false);
  });

  it("delegates addCategory to categoriesApi.createCategory", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();
    const newCategory: CategoryItem = {
      id: 25,
      name: "Dining",
      subcategories: [],
    };

    const spy = vi.spyOn(categoriesApi, "createCategory").mockResolvedValue(newCategory);

    const res = await store.addCategory("Expense", "Dining");
    expect(spy).toHaveBeenCalledWith({ type_id: 2, name: "Dining" });
    expect(res).toEqual(newCategory);
    expect(store.categories.some((c) => c.id === 25 && c.name === "Dining")).toBe(true);
  });

  it("delegates renameCategory and deleteCategory to categoriesApi", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const renamedItem: CategoryItem = {
      id: 10,
      name: "Primary Salary",
      subcategories: [
        { id: 100, name: "Primary Employer" },
        { id: 101, name: "Bonus" },
      ],
    };

    const renameSpy = vi.spyOn(categoriesApi, "updateCategory").mockResolvedValue(renamedItem);
    const renamed = await store.renameCategory(10, "Primary Salary");
    expect(renameSpy).toHaveBeenCalledWith(10, { name: "Primary Salary" });
    expect(renamed).toBe(true);
    expect(store.categories.find((c) => c.id === 10)?.name).toBe("Primary Salary");

    const deleteSpy = vi.spyOn(categoriesApi, "deleteCategory").mockResolvedValue(null);
    const deleted = await store.deleteCategory(10);
    expect(deleteSpy).toHaveBeenCalledWith(10);
    expect(deleted).toBe(true);
    expect(store.categories.find((c) => c.id === 10)).toBeUndefined();
  });

  it("delegates subcategory operations to categoriesApi", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const newSub: SubcategoryItem = { id: 123, name: "Stock Options" };
    const addSpy = vi.spyOn(categoriesApi, "createSubcategory").mockResolvedValue(newSub);

    const created = await store.addSubcategory(10, "Stock Options");
    expect(addSpy).toHaveBeenCalledWith({ category_id: 10, name: "Stock Options" });
    expect(created).toEqual(newSub);

    const renamedSub: SubcategoryItem = { id: 100, name: "Equity Awards" };
    const renameSpy = vi.spyOn(categoriesApi, "updateSubcategory").mockResolvedValue(renamedSub);
    const renamed = await store.renameSubcategory(100, "Equity Awards");
    expect(renameSpy).toHaveBeenCalledWith(100, { name: "Equity Awards" });
    expect(renamed).toBe(true);

    const deleteSpy = vi.spyOn(categoriesApi, "deleteSubcategory").mockResolvedValue(null);
    const deleted = await store.deleteSubcategory(100);
    expect(deleteSpy).toHaveBeenCalledWith(100);
    expect(deleted).toBe(true);
  });

  it("delegates resetDefaults to categoriesApi.resetDefaults and reloads", async () => {
    const store = new CategoryStore();
    let getCall = 0;
    vi.spyOn(categoriesApi, "getHierarchy").mockImplementation(async () => {
      getCall++;
      return getCall === 1 ? { types: [], colors: [] } : mockDefaults;
    });

    await store.load();
    expect(store.types).toHaveLength(0);

    const resetSpy = vi.spyOn(categoriesApi, "resetDefaults").mockResolvedValue(null);

    const success = await store.resetDefaults();
    expect(resetSpy).toHaveBeenCalled();
    expect(success).toBe(true);
    expect(store.types).toHaveLength(4);
    expect(store.categories).toHaveLength(8);
  });

  it("generates correct Sankey node and link structures from reactive state", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(mockDefaults);
    await store.load();

    const { nodes, links } = store.getSankeyData("All");
    expect(nodes.length).toBeGreaterThan(0);
    expect(links.length).toBeGreaterThan(0);

    const typeNodes = nodes.filter((n) => n.depth === 0);
    expect(typeNodes).toHaveLength(4);

    const nodeNames = new Set(nodes.map((n) => n.name));
    for (const link of links) {
      expect(nodeNames.has(link.source)).toBe(true);
      expect(nodeNames.has(link.target)).toBe(true);
      expect(link.value).toBeGreaterThan(0);
    }
  });

  it("filters Sankey data with custom arm ratios (40% Type->Cat, 60% Cat->Sub)", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(mockDefaults);
    await store.load();

    const { nodes } = store.getSankeyData("Expense");
    const typeNodes = nodes.filter((n) => n.level === "type");
    expect(typeNodes).toHaveLength(1);
    expect(typeNodes[0]?.depth).toBe(0);

    const catNodes = nodes.filter((n) => n.level === "category");
    expect(catNodes.every((n) => n.depth === 2)).toBe(true);

    const subNodes = nodes.filter((n) => n.level === "subcategory");
    expect(subNodes.every((n) => n.depth === 5)).toBe(true);
  });

  it("propagates errors when addType, addCategory, or addSubcategory fail", async () => {
    const store = new CategoryStore();

    vi.spyOn(categoriesApi, "createType").mockRejectedValue(
      new Error("Transaction type 'Income' already exists"),
    );
    await expect(store.addType("Income", 1)).rejects.toThrow(
      "Transaction type 'Income' already exists",
    );
    expect(store.error).toBeNull();

    vi.spyOn(categoriesApi, "createCategory").mockRejectedValue(
      new Error("Category 'Housing' already exists under type"),
    );
    await expect(store.addCategory("Expense", "Housing")).rejects.toThrow(
      "Category 'Housing' already exists under type",
    );
    expect(store.error).toBeNull();

    vi.spyOn(categoriesApi, "createSubcategory").mockRejectedValue(
      new Error("Subcategory 'Rent' already exists under category"),
    );
    await expect(store.addSubcategory(20, "Rent")).rejects.toThrow(
      "Subcategory 'Rent' already exists under category",
    );
    expect(store.error).toBeNull();
  });
});
