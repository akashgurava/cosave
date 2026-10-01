import { describe, it, expect, beforeAll } from "vitest";
import { api, FetchTransportAdapter } from "$lib/api";
import { categoriesApi } from "./api";

const isIntegration = process.env.TEST_INTEGRATION === "1" || !!process.env.TEST_API_URL;
const describeIntegration = isIntegration ? describe : describe.skip;

describeIntegration("Categories Live API Integration (Full-Stack Axum Roundtrip)", () => {
  const baseUrl = process.env.TEST_API_URL || "http://127.0.0.1:5171";
  const fetchTransport = new FetchTransportAdapter(baseUrl);

  beforeAll(() => {
    api.setTransport(fetchTransport);
  });

  describe("Read Operations", () => {
    it("fetches and decodes real hierarchy from live backend", async () => {
      const hierarchy = await categoriesApi.getHierarchy();
      expect(hierarchy.types.length).toBeGreaterThanOrEqual(4);
      expect(hierarchy.colors.length).toBe(12);

      const income = hierarchy.types.find((t) => t.name === "Income");
      expect(income).toBeDefined();
      expect(income?.id).toBeTypeOf("number");
      expect(income?.categories.length).toBeGreaterThanOrEqual(1);

      const salary = income?.categories.find((c) => c.name === "Salary & Wages");
      expect(salary).toBeDefined();
      expect(salary?.id).toBeTypeOf("number");
      expect(salary?.subcategories.length).toBeGreaterThanOrEqual(1);
    });

    it("fetches and decodes real palette colors from live backend", async () => {
      const colors = await categoriesApi.getColors();
      expect(colors).toHaveLength(12);
      expect(colors[0]?.id).toBeTypeOf("number");
      expect(colors[0]?.name).toBe("Emerald");
      expect(colors[0]?.hex).toBe("#10b981");
    });
  });

  describe("Authenticated Lifecycle Operations", () => {
    beforeAll(async () => {
      const testUser = `test_admin_${Date.now()}`;
      try {
        const regRes = await fetch(`${baseUrl}/api/v1/auth/register`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ username: testUser, password: "Password123!" }),
        });
        const cookie = regRes.headers.get("set-cookie");
        if (cookie) {
          const token = cookie.split(";")[0]?.split("=")[1];
          if (token) fetchTransport.setCookie("cosave_session", token);
        }
      } catch {
        const loginRes = await fetch(`${baseUrl}/api/v1/auth/login`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ username: "admin", password: "password123" }),
        });
        const cookie = loginRes.headers.get("set-cookie");
        if (cookie) {
          const token = cookie.split(";")[0]?.split("=")[1];
          if (token) fetchTransport.setCookie("cosave_session", token);
        }
      }
    });

    it("executes complete taxonomy CRUD lifecycle with CQS validation", async () => {
      // 1. Create a new transaction type
      const uniqueName = `IntegrationType_${Date.now()}`;
      const createdType = await categoriesApi.createType({
        name: uniqueName,
        color_id: 6,
      });
      expect(createdType.id).toBeTypeOf("number");
      expect(createdType.color_id).toBe(6);

      // 2. Update type color (CQS ack)
      const colorAck = await categoriesApi.updateTypeColor(createdType.id, { color_id: 4 });
      expect(colorAck).toBeNull();

      // 3. Create a category under this type
      const createdCat = await categoriesApi.createCategory({
        type_id: createdType.id,
        name: `IntegrationCat_${Date.now()}`,
      });
      expect(createdCat.id).toBeTypeOf("number");

      // 4. Rename category
      const renamedCat = await categoriesApi.updateCategory(createdCat.id, {
        name: `RenamedCat_${Date.now()}`,
      });
      expect(renamedCat.id).toBe(createdCat.id);

      // 5. Create a subcategory
      const createdSub = await categoriesApi.createSubcategory({
        category_id: createdCat.id,
        name: `IntegrationSub_${Date.now()}`,
      });
      expect(createdSub.id).toBeTypeOf("number");

      // 6. Rename subcategory
      const renamedSub = await categoriesApi.updateSubcategory(createdSub.id, {
        name: `RenamedSub_${Date.now()}`,
      });
      expect(renamedSub.id).toBe(createdSub.id);

      // 7. Delete subcategory (CQS ack)
      const delSubAck = await categoriesApi.deleteSubcategory(createdSub.id);
      expect(delSubAck).toBeNull();

      // 8. Delete category (CQS ack)
      const delCatAck = await categoriesApi.deleteCategory(createdCat.id);
      expect(delCatAck).toBeNull();

      // 9. Delete type (CQS ack)
      const delTypeAck = await categoriesApi.deleteType(createdType.id);
      expect(delTypeAck).toBeNull();
    });

    it("resets taxonomy defaults and confirms hierarchy state", async () => {
      const resetAck = await categoriesApi.resetDefaults();
      expect(resetAck).toBeNull();

      const hierarchy = await categoriesApi.getHierarchy();
      expect(hierarchy.types.length).toBe(4);
    });
  });
});
