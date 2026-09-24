import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  api,
  ApiError,
  Code,
  ContractViolationError,
  MemoryTransportAdapter,
  Status,
} from "$lib/api";
import { categoriesApi } from "./api";
import { CategoryStore } from "./store";
import { PRESET_COLORS, type CategoryHierarchyResponse } from "./types";

const mockInitialHierarchy: CategoryHierarchyResponse = {
  types: [
    { id: "type-income", name: "Income", color: "#10b981" },
    { id: "type-expense", name: "Expense", color: "#f43f5e" },
  ],
  categories: [
    {
      id: "cat-salary",
      name: "Salary",
      type: "Income",
      subcategories: [{ id: "sub-salary-base", name: "Base Salary" }],
    },
    {
      id: "cat-housing",
      name: "Housing",
      type: "Expense",
      subcategories: [{ id: "sub-housing-rent", name: "Rent" }],
    },
  ],
  colors: [...PRESET_COLORS],
};

describe("Categories API & Store Integration (Contract Seam & Envelope Decoders)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  describe("categoriesApi.getHierarchy", () => {
    it("fetches and decodes full category hierarchy correctly", async () => {
      memoryTransport.on("GET", "/api/v1/categories", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockInitialHierarchy,
      }));

      const hierarchy = await categoriesApi.getHierarchy();
      expect(hierarchy.types).toHaveLength(2);
      expect(hierarchy.types[0].name).toBe("Income");
      expect(hierarchy.categories).toHaveLength(2);
      expect(hierarchy.categories[0].subcategories[0].name).toBe("Base Salary");
    });

    it("throws ContractViolationError when hierarchy payload is malformed", async () => {
      memoryTransport.on("GET", "/api/v1/categories", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { types: "not-an-array", categories: [] },
      }));

      await expect(categoriesApi.getHierarchy()).rejects.toThrow(ContractViolationError);
    });
  });

  describe("Transaction Type Operations", () => {
    it("creates a new transaction type and parses TransactionTypeItem", async () => {
      memoryTransport.on("POST", "/api/v1/categories/types", (req) => {
        expect(req.headers["Content-Type"]).toBe("application/json");
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Investment");
        expect(body.color).toBe("#3b82f6");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "type-inv",
            name: "Investment",
            color: "#3b82f6",
          },
        };
      });

      const created = await categoriesApi.createType({
        name: "Investment",
        color: "#3b82f6",
      });

      expect(created.id).toBe("type-inv");
      expect(created.name).toBe("Investment");
      expect(created.color).toBe("#3b82f6");
    });

    it("updates type color with interpolated path parameter", async () => {
      memoryTransport.on("PATCH", "/api/v1/categories/types/type-income/color", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.color).toBe("#059669");

        const updated = structuredClone(mockInitialHierarchy);
        updated.types[0].color = "#059669";
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.updateTypeColor("type-income", "#059669");
      expect(hierarchy.types[0].color).toBe("#059669");
    });

    it("deletes type with interpolated path parameter", async () => {
      memoryTransport.on("DELETE", "/api/v1/categories/types/type-expense", () => {
        const updated = structuredClone(mockInitialHierarchy);
        updated.types = updated.types.filter((t) => t.id !== "type-expense");
        updated.categories = updated.categories.filter((c) => c.type !== "Expense");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.deleteType("type-expense");
      expect(hierarchy.types).toHaveLength(1);
      expect(hierarchy.categories).toHaveLength(1);
    });
  });

  describe("Category Operations", () => {
    it("creates a category under a type and returns CategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/categories", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Freelance");
        expect(body.type_name).toBe("Income");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "cat-freelance",
            name: "Freelance",
            type: "Income",
            subcategories: [],
          },
        };
      });

      const cat = await categoriesApi.createCategory({
        type_name: "Income",
        name: "Freelance",
      });

      expect(cat.id).toBe("cat-freelance");
      expect(cat.name).toBe("Freelance");
      expect(cat.type).toBe("Income");
      expect(cat.subcategories).toEqual([]);
    });

    it("updates category name and returns refreshed hierarchy", async () => {
      memoryTransport.on("PATCH", "/api/v1/categories/cat-salary", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Primary Salary");

        const updated = structuredClone(mockInitialHierarchy);
        updated.categories[0].name = "Primary Salary";
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.updateCategory("cat-salary", "Primary Salary");
      expect(hierarchy.categories[0].name).toBe("Primary Salary");
    });

    it("deletes category and returns refreshed hierarchy", async () => {
      memoryTransport.on("DELETE", "/api/v1/categories/cat-housing", () => {
        const updated = structuredClone(mockInitialHierarchy);
        updated.categories = updated.categories.filter((c) => c.id !== "cat-housing");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.deleteCategory("cat-housing");
      expect(hierarchy.categories).toHaveLength(1);
      expect(hierarchy.categories[0].id).toBe("cat-salary");
    });
  });

  describe("Subcategory Operations", () => {
    it("creates a subcategory under a category and returns SubcategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/categories/subcategories", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.category_id).toBe("cat-salary");
        expect(body.name).toBe("Bonus");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: "sub-salary-bonus",
            name: "Bonus",
          },
        };
      });

      const sub = await categoriesApi.createSubcategory({
        category_id: "cat-salary",
        name: "Bonus",
      });

      expect(sub.id).toBe("sub-salary-bonus");
      expect(sub.name).toBe("Bonus");
    });

    it("updates subcategory name and returns refreshed hierarchy", async () => {
      memoryTransport.on("PATCH", "/api/v1/categories/subcategories/sub-salary-base", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Base Monthly Salary");

        const updated = structuredClone(mockInitialHierarchy);
        updated.categories[0].subcategories[0].name = "Base Monthly Salary";
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.updateSubcategory(
        "sub-salary-base",
        "Base Monthly Salary",
      );
      expect(hierarchy.categories[0].subcategories[0].name).toBe("Base Monthly Salary");
    });

    it("deletes subcategory and returns refreshed hierarchy", async () => {
      memoryTransport.on("DELETE", "/api/v1/categories/subcategories/sub-salary-base", () => {
        const updated = structuredClone(mockInitialHierarchy);
        updated.categories[0].subcategories = [];
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const hierarchy = await categoriesApi.deleteSubcategory("sub-salary-base");
      expect(hierarchy.categories[0].subcategories).toHaveLength(0);
    });
  });

  describe("categoriesApi.resetDefaults", () => {
    it("posts to reset endpoint and decodes default hierarchy", async () => {
      let resetCalled = false;
      memoryTransport.on("POST", "/api/v1/categories/reset", () => {
        resetCalled = true;
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: mockInitialHierarchy,
        };
      });

      const hierarchy = await categoriesApi.resetDefaults();
      expect(resetCalled).toBe(true);
      expect(hierarchy.types).toHaveLength(2);
      expect(hierarchy.categories).toHaveLength(2);
    });

    it("throws ApiError when reset endpoint fails with server error", async () => {
      memoryTransport.on("POST", "/api/v1/categories/reset", () => ({
        code: 500,
        status: "INTERNAL_ERROR",
        data: null,
      }));

      let err: ApiError | null = null;
      try {
        await categoriesApi.resetDefaults();
      } catch (e) {
        if (e instanceof ApiError) {
          err = e;
        }
      }

      expect(err).toBeInstanceOf(ApiError);
      expect(err?.httpStatus).toBe(500);
      expect(err?.apiStatus).toBe("INTERNAL_ERROR");
    });
  });

  describe("CategoryStore Integration with Real API Transport", () => {
    it("loads and populates store reactively through categoriesApi", async () => {
      memoryTransport.on("GET", "/api/v1/categories", () => ({
        code: 0,
        status: "OK",
        data: mockInitialHierarchy,
      }));

      const store = new CategoryStore();
      expect(store.isLoaded).toBe(false);
      expect(store.types).toHaveLength(0);

      await store.load();

      expect(store.isLoaded).toBe(true);
      expect(store.types).toHaveLength(2);
      expect(store.categories).toHaveLength(2);
    });

    it("resets categories back to defaults through real API call", async () => {
      memoryTransport.on("GET", "/api/v1/categories", () => ({
        code: 0,
        status: "OK",
        data: { types: [], categories: [] },
      }));

      memoryTransport.on("POST", "/api/v1/categories/reset", () => ({
        code: 0,
        status: "OK",
        data: mockInitialHierarchy,
      }));

      const store = new CategoryStore();
      await store.load();
      expect(store.types).toHaveLength(0);

      await store.resetDefaults();
      expect(store.types).toHaveLength(2);
      expect(store.categories).toHaveLength(2);
    });
  });
});
