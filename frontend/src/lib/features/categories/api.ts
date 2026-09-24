import { api, ContractViolationError } from "$lib/api";
import {
  parseCategoryHierarchyResponse,
  parseCategoryItem,
  parseColorOption,
  parseSubcategoryItem,
  parseTransactionTypeItem,
  type CategoryHierarchyResponse,
  type CategoryItem,
  type ColorOption,
  type CreateCategoryPayload,
  type CreateSubcategoryPayload,
  type CreateTypePayload,
  type SubcategoryItem,
  type TransactionTypeItem,
} from "./types";

/**
 * Category & Transaction Hierarchy API service functions with runtime schema contract enforcement.
 * The Rust backend is the authoritative Single Source of Truth (SSOT).
 */
export const categoriesApi = {
  getHierarchy(): Promise<CategoryHierarchyResponse> {
    return api.get<CategoryHierarchyResponse>("/api/v1/categories", {
      schema: parseCategoryHierarchyResponse,
    });
  },

  getColors(): Promise<ColorOption[]> {
    return api.get<ColorOption[]>("/api/v1/categories/colors", {
      schema: (raw) => {
        if (!Array.isArray(raw)) {
          throw new ContractViolationError("Expected array of colors", raw);
        }
        return raw.map(parseColorOption);
      },
    });
  },

  createType(payload: CreateTypePayload): Promise<TransactionTypeItem> {
    return api.post<TransactionTypeItem>("/api/v1/categories/types", payload, {
      schema: parseTransactionTypeItem,
    });
  },

  updateTypeColor(id: string, color: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/types/:id/color",
      { color },
      { pathParams: { id }, schema: parseCategoryHierarchyResponse },
    );
  },

  deleteType(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/types/:id", {
      pathParams: { id },
      schema: parseCategoryHierarchyResponse,
    });
  },

  createCategory(payload: CreateCategoryPayload): Promise<CategoryItem> {
    return api.post<CategoryItem>("/api/v1/categories", payload, {
      schema: parseCategoryItem,
    });
  },

  updateCategory(id: string, name: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/:id",
      { name },
      { pathParams: { id }, schema: parseCategoryHierarchyResponse },
    );
  },

  deleteCategory(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/:id", {
      pathParams: { id },
      schema: parseCategoryHierarchyResponse,
    });
  },

  createSubcategory(payload: CreateSubcategoryPayload): Promise<SubcategoryItem> {
    return api.post<SubcategoryItem>("/api/v1/categories/subcategories", payload, {
      schema: parseSubcategoryItem,
    });
  },

  updateSubcategory(id: string, name: string): Promise<CategoryHierarchyResponse> {
    return api.patch<CategoryHierarchyResponse>(
      "/api/v1/categories/subcategories/:id",
      { name },
      { pathParams: { id }, schema: parseCategoryHierarchyResponse },
    );
  },

  deleteSubcategory(id: string): Promise<CategoryHierarchyResponse> {
    return api.delete<CategoryHierarchyResponse>("/api/v1/categories/subcategories/:id", {
      pathParams: { id },
      schema: parseCategoryHierarchyResponse,
    });
  },

  resetDefaults(): Promise<CategoryHierarchyResponse> {
    return api.post<CategoryHierarchyResponse>("/api/v1/categories/reset", undefined, {
      schema: parseCategoryHierarchyResponse,
    });
  },
};
