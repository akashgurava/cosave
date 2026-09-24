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
    { id: "type-income", name: "Income", color: "#10b981" },
    { id: "type-expense", name: "Expense", color: "#f43f5e" },
    { id: "type-transfer", name: "Transfer", color: "#71717a" },
    { id: "type-invest", name: "Invest", color: "#3b82f6" },
  ],
  categories: [
    {
      id: "cat-inc-salary",
      name: "Salary",
      type: "Income",
      subcategories: [
        { id: "sub-inc-primary", name: "Primary Employer" },
        { id: "sub-inc-bonus", name: "Bonus" },
      ],
    },
    {
      id: "cat-inc-freelance",
      name: "Freelance",
      type: "Income",
      subcategories: [
        { id: "sub-inc-consulting", name: "Consulting" },
        { id: "sub-inc-retainers", name: "Retainers" },
      ],
    },
    {
      id: "cat-exp-housing",
      name: "Housing",
      type: "Expense",
      subcategories: [
        { id: "sub-exp-rent", name: "Rent" },
        { id: "sub-exp-utilities", name: "Utilities" },
      ],
    },
    {
      id: "cat-exp-food",
      name: "Food",
      type: "Expense",
      subcategories: [
        { id: "sub-exp-groceries", name: "Groceries" },
        { id: "sub-exp-dining", name: "Dining Out" },
      ],
    },
    {
      id: "cat-exp-transport",
      name: "Transport",
      type: "Expense",
      subcategories: [
        { id: "sub-exp-fuel", name: "Fuel" },
        { id: "sub-exp-transit", name: "Public Transit" },
      ],
    },
    {
      id: "cat-exp-personal",
      name: "Personal",
      type: "Expense",
      subcategories: [{ id: "sub-exp-fitness", name: "Gym & Fitness" }],
    },
    {
      id: "cat-trf-internal",
      name: "Internal",
      type: "Transfer",
      subcategories: [
        { id: "sub-trf-checking", name: "Checking to Savings" },
        { id: "sub-trf-emergency", name: "Emergency Fund" },
      ],
    },
    {
      id: "cat-inv-equities",
      name: "Equities",
      type: "Invest",
      subcategories: [{ id: "sub-inv-etf", name: "Index ETFs" }],
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
      id: "type-savings",
      name: "Savings",
      color: "#f59e0b",
    };

    const spy = vi.spyOn(categoriesApi, "createType").mockResolvedValue(createdType);

    const res = await store.addType("Savings", "#f59e0b");
    expect(spy).toHaveBeenCalledWith({ name: "Savings", color: "#f59e0b" });
    expect(res).toEqual(createdType);
    expect(store.types).toContainEqual(createdType);
  });

  it("delegates updateTypeColor to categoriesApi.updateTypeColor", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const updatedMock = structuredClone(mockDefaults);
    const incomeType = updatedMock.types.find((t) => t.name === "Income");
    if (incomeType) incomeType.color = "#f59e0b";

    const spy = vi.spyOn(categoriesApi, "updateTypeColor").mockResolvedValue(updatedMock);

    const success = await store.updateTypeColor("Income", "#f59e0b");
    expect(spy).toHaveBeenCalledWith("type-income", "#f59e0b");
    expect(success).toBe(true);
    expect(store.getType("Income")?.color).toBe("#f59e0b");
  });

  it("delegates deleteType to categoriesApi.deleteType and cascades local state", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const deletedMock = structuredClone(mockDefaults);
    deletedMock.types = deletedMock.types.filter((t) => t.name !== "Income");
    deletedMock.categories = deletedMock.categories.filter((c) => c.type !== "Income");

    const spy = vi.spyOn(categoriesApi, "deleteType").mockResolvedValue(deletedMock);

    const success = await store.deleteType("Income");
    expect(spy).toHaveBeenCalledWith("type-income");
    expect(success).toBe(true);
    expect(store.getType("Income")).toBeNull();
    expect(store.categories.some((c) => c.type === "Income")).toBe(false);
  });

  it("delegates addCategory to categoriesApi.createCategory", async () => {
    const store = new CategoryStore();
    const newCategory: CategoryItem = {
      id: "cat-dining",
      name: "Dining",
      type: "Expense",
      subcategories: [],
    };

    const spy = vi.spyOn(categoriesApi, "createCategory").mockResolvedValue(newCategory);

    const res = await store.addCategory("Expense", "Dining");
    expect(spy).toHaveBeenCalledWith({ type_name: "Expense", name: "Dining" });
    expect(res).toEqual(newCategory);
    expect(store.categories).toContainEqual(newCategory);
  });

  it("delegates renameCategory and deleteCategory to categoriesApi", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const renamedMock = structuredClone(mockDefaults);
    const cat = renamedMock.categories.find((c) => c.id === "cat-inc-salary");
    if (cat) cat.name = "Primary Salary";

    const renameSpy = vi.spyOn(categoriesApi, "updateCategory").mockResolvedValue(renamedMock);
    const renamed = await store.renameCategory("cat-inc-salary", "Primary Salary");
    expect(renameSpy).toHaveBeenCalledWith("cat-inc-salary", "Primary Salary");
    expect(renamed).toBe(true);
    expect(store.categories.find((c) => c.id === "cat-inc-salary")?.name).toBe("Primary Salary");

    const deletedMock = structuredClone(mockDefaults);
    deletedMock.categories = deletedMock.categories.filter((c) => c.id !== "cat-inc-salary");

    const deleteSpy = vi.spyOn(categoriesApi, "deleteCategory").mockResolvedValue(deletedMock);
    const deleted = await store.deleteCategory("cat-inc-salary");
    expect(deleteSpy).toHaveBeenCalledWith("cat-inc-salary");
    expect(deleted).toBe(true);
    expect(store.categories.find((c) => c.id === "cat-inc-salary")).toBeUndefined();
  });

  it("delegates subcategory operations to categoriesApi with deepened signatures", async () => {
    const store = new CategoryStore();
    vi.spyOn(categoriesApi, "getHierarchy").mockResolvedValue(structuredClone(mockDefaults));
    await store.load();

    const newSub: SubcategoryItem = { id: "sub-123", name: "Stock Options" };
    const addSpy = vi.spyOn(categoriesApi, "createSubcategory").mockResolvedValue(newSub);

    const created = await store.addSubcategory("cat-inc-salary", "Stock Options");
    expect(addSpy).toHaveBeenCalledWith({ category_id: "cat-inc-salary", name: "Stock Options" });
    expect(created).toEqual(newSub);

    const renamedSubMock = structuredClone(mockDefaults);
    const targetCat = renamedSubMock.categories.find((c) => c.id === "cat-inc-salary");
    if (targetCat) {
      targetCat.subcategories.push({ id: "sub-123", name: "Equity Awards" });
    }

    const renameSpy = vi
      .spyOn(categoriesApi, "updateSubcategory")
      .mockResolvedValue(renamedSubMock);
    const renamed = await store.renameSubcategory("sub-123", "Equity Awards");
    expect(renameSpy).toHaveBeenCalledWith("sub-123", "Equity Awards");
    expect(renamed).toBe(true);

    const deletedSubMock = structuredClone(mockDefaults);
    const deleteSpy = vi
      .spyOn(categoriesApi, "deleteSubcategory")
      .mockResolvedValue(deletedSubMock);
    const deleted = await store.deleteSubcategory("sub-123");
    expect(deleteSpy).toHaveBeenCalledWith("sub-123");
    expect(deleted).toBe(true);
  });

  it("delegates resetDefaults to categoriesApi.resetDefaults", async () => {
    const store = new CategoryStore();
    const resetSpy = vi.spyOn(categoriesApi, "resetDefaults").mockResolvedValue(mockDefaults);

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
    expect(typeNodes[0].depth).toBe(0);

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
    await expect(store.addType("Income", "#10b981")).rejects.toThrow(
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
    await expect(store.addSubcategory("cat-exp-housing", "Rent")).rejects.toThrow(
      "Subcategory 'Rent' already exists under category",
    );
    expect(store.error).toBeNull();
  });
});
