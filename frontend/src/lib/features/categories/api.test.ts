import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { api, ApiError, Code, ContractViolationError, Status } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { categoriesApi } from "./api";
import { CategoryStore } from "./store.svelte";
import type { CategoryHierarchyResponse, Color } from "./types";
import { CategoryId, ColorId, SubcategoryId, TypeId } from "$lib/types";

const mockColors: readonly Color[] = [
  { id: ColorId(1), name: "Emerald", hex: "#10b981" },
  { id: ColorId(2), name: "Rose", hex: "#f43f5e" },
  { id: ColorId(3), name: "Grey", hex: "#71717a" },
  { id: ColorId(4), name: "Blue", hex: "#3b82f6" },
  { id: ColorId(5), name: "Amber", hex: "#f59e0b" },
  { id: ColorId(6), name: "Violet", hex: "#8b5cf6" },
];

const mockInitialHierarchy: CategoryHierarchyResponse = {
  types: [
    {
      id: TypeId(1),
      name: "Income",
      color: "#10b981",
      colorId: ColorId(1),
      categories: [
        {
          id: CategoryId(10),
          name: "Salary",
          subcategories: [{ id: SubcategoryId(100), name: "Base Salary" }],
        },
      ],
    },
    {
      id: TypeId(2),
      name: "Expense",
      color: "#f43f5e",
      colorId: ColorId(2),
      categories: [
        {
          id: CategoryId(20),
          name: "Housing",
          subcategories: [{ id: SubcategoryId(200), name: "Rent" }],
        },
      ],
    },
  ],
  colors: [...mockColors],
};

describe("Categories API & Store Integration (Contract Seam & Envelope Decoders)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport();
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
      const firstType = hierarchy.types[0];
      expect(firstType).toBeDefined();
      if (firstType !== undefined) {
        expect(firstType.name).toBe("Income");
        expect(firstType.categories).toHaveLength(1);
        const firstCat = firstType.categories[0];
        expect(firstCat).toBeDefined();
        if (firstCat !== undefined) {
          const firstSub = firstCat.subcategories[0];
          expect(firstSub).toBeDefined();
          if (firstSub !== undefined) {
            expect(firstSub.name).toBe("Base Salary");
          }
        }
      }
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
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
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
        colorId: ColorId(4),
      });

      expect(created.id).toBe(3);
      expect(created.name).toBe("Investment");
      expect(created.color).toBe("#3b82f6");
      expect(created.colorId).toBe(4);
    });

    it("updates type color with interpolated path parameter and CQS acknowledgement", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/types/1/color", (req) => {
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
        expect(body.colorId).toBe(6);

        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const ack = await categoriesApi.updateTypeColor(TypeId(1), { colorId: ColorId(6) });
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

      const ack = await categoriesApi.deleteType(TypeId(2));
      expect(ack).toBeNull();
    });
  });

  describe("Category Operations", () => {
    it("creates a category under a type and returns CategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/config/categories", (req) => {
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
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
        typeId: TypeId(1),
        name: "Freelance",
      });

      expect(cat.id).toBe(11);
      expect(cat.name).toBe("Freelance");
      expect(cat.subcategories).toEqual([]);
    });

    it("updates category name and returns renamed CategoryItem", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/10", (req) => {
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
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

      const updated = await categoriesApi.updateCategory(CategoryId(10), {
        name: "Primary Salary",
      });
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

      const ack = await categoriesApi.deleteCategory(CategoryId(20));
      expect(ack).toBeNull();
    });
  });

  describe("Subcategory Operations", () => {
    it("creates a subcategory under a category and returns SubcategoryItem", async () => {
      memoryTransport.on("POST", "/api/v1/config/categories/subcategories", (req) => {
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
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
        categoryId: CategoryId(10),
        name: "Bonus",
      });

      expect(sub.id).toBe(101);
      expect(sub.name).toBe("Bonus");
    });

    it("updates subcategory name and returns renamed SubcategoryItem", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/categories/subcategories/100", (req) => {
        const body = req.body !== undefined ? JSON.parse(req.body) : {};
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

      const updated = await categoriesApi.updateSubcategory(SubcategoryId(100), {
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

      const ack = await categoriesApi.deleteSubcategory(SubcategoryId(100));
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
      if (err !== null) {
        expect(err.httpStatus).toBe(500);
        expect(err.apiStatus).toBe("INTERNAL_ERROR");
      }
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
      expect(store.isSuccess).toBe(false);
      expect(store.types).toHaveLength(0);

      await store.load();

      expect(store.isSuccess).toBe(true);
      expect(store.types).toHaveLength(2);
      expect(store.categories).toHaveLength(2);
      const firstCat = store.categories[0];
      expect(firstCat).toBeDefined();
      if (firstCat !== undefined) {
        expect(firstCat.name).toBe("Salary");
      }
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
