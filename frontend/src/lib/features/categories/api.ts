import { api } from "$lib/api";
import type {
  CategoryHierarchyResponse,
  CategoryItem,
  CreateCategoryPayload,
  CreateSubcategoryPayload,
  CreateTypePayload,
  SubcategoryItem,
  TransactionTypeItem,
} from "./types";

/**
 * Category & Transaction Hierarchy API service functions.
 * The Rust backend is the authoritative Single Source of Truth (SSOT).
 */
export const categoriesApi = {
  getHierarchy(): Promise<CategoryHierarchyResponse> {
    return api.get<CategoryHierarchyResponse>("/api/v1/categories");
  },

  createType(payload: CreateTypePayload): Promise<TransactionTypeItem> {
    return api.post<TransactionTypeItem>("/api/v1/categories/types", payload);
  },

  updateTypeColor(id: string, color: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/types/:id/color",
      { color },
      { pathParams: { id } },
    );
  },

  deleteType(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/types/:id", {
      pathParams: { id },
    });
  },

  createCategory(payload: CreateCategoryPayload): Promise<CategoryItem> {
    return api.post<CategoryItem>("/api/v1/categories", payload);
  },

  updateCategory(id: string, name: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/:id",
      { name },
      { pathParams: { id } },
    );
  },

  deleteCategory(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/:id", {
      pathParams: { id },
    });
  },

  createSubcategory(payload: CreateSubcategoryPayload): Promise<SubcategoryItem> {
    return api.post<SubcategoryItem>("/api/v1/categories/subcategories", payload);
  },

  updateSubcategory(id: string, name: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/subcategories/:id",
      { name },
      { pathParams: { id } },
    );
  },

  deleteSubcategory(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/subcategories/:id", {
      pathParams: { id },
    });
  },

  resetDefaults(): Promise<CategoryHierarchyResponse> {
    return api.post<CategoryHierarchyResponse>("/api/v1/categories/reset");
  },
};
