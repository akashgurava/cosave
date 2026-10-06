import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, Code, ContractViolationError, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { categoriesApi } from "./api";
import { CategoryStore } from "./store";
import { PRESET_COLORS, type CategoryHierarchyResponse } from "./types";

const mockInitialHierarchy: CategoryHierarchyResponse = {
  types: [
    {
      id: 1,
      name: "Income",
      color: "#10b981",
      colorId: 1,
      categories: [
        {
          id: 10,
          name: "Salary",
          subcategories: [{ id: 100, name: "Base Salary" }],
        },
      ],
    },
    {
      id: 2,
      name: "Expense",
      color: "#f43f5e",
      colorId: 2,
      categories: [
        {
          id: 20,
          name: "Housing",
          subcategories: [{ id: 200, name: "Rent" }],
        },
      ],
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
      memoryTransport.on("GET", "/api/v1/config/hierarchy", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockInitialHierarchy,
      }));

      const hierarchy = await categoriesApi.getHierarchy();
      expect(hierarchy.types).toHaveLength(2);
      expect(hierarchy.types[0]?.name).toBe("Income");
      expect(hierarchy.types[0]?.categories).toHaveLength(1);
      expect(hierarchy.types[0]?.categories[0]?.subcategories[0]?.name).toBe("Base Salary");
    });

    it("throws ContractViolationError when hierarchy payload is malformed", async () => {
      memoryTransport.on("GET", "/api/v1/config/hierarchy", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { types: "not-an-array" },
      }));

      await expect(categoriesApi.getHierarchy()).rejects.toThrow(ContractViolationError);
    });
  });

  describe("Transaction Type Operations", () => {
    it("creates a new transaction type and parses TransactionTypeItem", async () => {
      memoryTransport.on("POST", "/api/v1/config/categories/types", (req) => {
        expect(req.headers["Content-Type"]).toBe("application/json");
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Investment");
        expect(body.colorId).toBe(4);

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: 3,
            name: "Investment",
            color: "#3b82f6",
            colorId: 4,
            categories: [],
          },
        };
      });

      const created = await categoriesApi.createType({
        name: "Investment",
        colorId: 4,
      });

      expect(created.id).toBe(3);
      expect(created.name).toBe("Investment");
      expect(created.color).toBe("#3b82f6");
      expect(created.colorId).toBe(4);
    });

    it("updates type color with interpolated path parameter and CQS acknowledgement", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/types/1/color", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.colorId).toBe(6);

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.updateTypeColor(1, { colorId: 6 });
      expect(ack).toBeNull();
    });

    it("deletes type with interpolated path parameter and CQS acknowledgement", async () => {
      memoryTransport.on("DELETE", "/api/v1/config/categories/types/2", () => {
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.deleteType(2);
      expect(ack).toBeNull();
    });
  });

  describe("Category Operations", () => {
    it("creates a category under a type and returns CategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/config/categories", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Freelance");
        expect(body.typeId).toBe(1);

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: 11,
            name: "Freelance",
            subcategories: [],
          },
        };
      });

      const cat = await categoriesApi.createCategory({
        typeId: 1,
        name: "Freelance",
      });

      expect(cat.id).toBe(11);
      expect(cat.name).toBe("Freelance");
      expect(cat.subcategories).toEqual([]);
    });

    it("updates category name and returns renamed CategoryItem", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/10", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Primary Salary");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: 10,
            name: "Primary Salary",
            subcategories: [{ id: 100, name: "Base Salary" }],
          },
        };
      });

      const updated = await categoriesApi.updateCategory(10, { name: "Primary Salary" });
      expect(updated.id).toBe(10);
      expect(updated.name).toBe("Primary Salary");
    });

    it("deletes category and returns CQS acknowledgement", async () => {
      memoryTransport.on("DELETE", "/api/v1/config/categories/20", () => {
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.deleteCategory(20);
      expect(ack).toBeNull();
    });
  });

  describe("Subcategory Operations", () => {
    it("creates a subcategory under a category and returns SubcategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/config/categories/subcategories", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.categoryId).toBe(10);
        expect(body.name).toBe("Bonus");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: 101,
            name: "Bonus",
          },
        };
      });

      const sub = await categoriesApi.createSubcategory({
        categoryId: 10,
        name: "Bonus",
      });

      expect(sub.id).toBe(101);
      expect(sub.name).toBe("Bonus");
    });

    it("updates subcategory name and returns renamed SubcategoryItem", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/subcategories/100", (req) => {
        const body = JSON.parse(req.body ?? "{}");
        expect(body.name).toBe("Base Monthly Salary");

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: {
            id: 100,
            name: "Base Monthly Salary",
          },
        };
      });

      const updated = await categoriesApi.updateSubcategory(100, {
        name: "Base Monthly Salary",
      });
      expect(updated.id).toBe(100);
      expect(updated.name).toBe("Base Monthly Salary");
    });

    it("deletes subcategory and returns CQS acknowledgement", async () => {
      memoryTransport.on("DELETE", "/api/v1/config/categories/subcategories/100", () => {
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.deleteSubcategory(100);
      expect(ack).toBeNull();
    });
  });

  describe("categoriesApi.resetDefaults", () => {
    it("posts to reset endpoint and receives CQS acknowledgement", async () => {
      let resetCalled = false;
      memoryTransport.on("POST", "/api/v1/config/hierarchy/reset", () => {
        resetCalled = true;
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.resetDefaults();
      expect(resetCalled).toBe(true);
      expect(ack).toBeNull();
    });

    it("throws ApiError when reset endpoint fails with server error", async () => {
      memoryTransport.on("POST", "/api/v1/config/hierarchy/reset", () => ({
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
      memoryTransport.on("GET", "/api/v1/config/hierarchy", () => ({
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
      expect(store.categories[0]?.name).toBe("Salary");
    });

    it("resets categories back to defaults through real API call and reloads", async () => {
      let getCallCount = 0;
      memoryTransport.on("GET", "/api/v1/config/hierarchy", () => {
        getCallCount++;
        return {
          code: 0,
          status: "OK",
          data: getCallCount === 1 ? { types: [], colors: [] } : mockInitialHierarchy,
        };
      });

      memoryTransport.on("POST", "/api/v1/config/hierarchy/reset", () => ({
        code: 0,
        status: "OK",
        data: null,
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
