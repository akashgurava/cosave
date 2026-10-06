/**
 * Category & Hierarchy API client functions with runtime schema contract enforcement.
 *
 * Dispatches requests to backend endpoints mounted under `/api/v1/config`,
 * wrapping payloads in strongly-typed models and verifying responses against
 * the Rust backend SSOT.
 */

import { api } from "$lib/api";
import { ContractViolationError, parseNull } from "$lib/api/contracts";
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
  type UpdateNamePayload,
  type UpdateTypeColorPayload,
} from "./types";

export const categoriesApi = {
  getHierarchy(): Promise<CategoryHierarchyResponse> {
    return api.get<CategoryHierarchyResponse>("/api/v1/config/hierarchy", {
      schema: parseCategoryHierarchyResponse,
    });
  },

  getColors(): Promise<ColorOption[]> {
    return api.get<ColorOption[]>("/api/v1/config/categories/colors", {
      schema: (raw) => {
        if (Array.isArray(raw) === false) {
          throw new ContractViolationError("Expected array of colors", raw);
        }
        return raw.map(parseColorOption);
      },
    });
  },

  createType(payload: CreateTypePayload): Promise<TransactionTypeItem> {
    return api.post<TransactionTypeItem>("/api/v1/config/categories/types", payload, {
      schema: parseTransactionTypeItem,
    });
  },

  updateTypeColor(id: number, payload: UpdateTypeColorPayload): Promise<null> {
    return api.patch<null>("/api/v1/config/categories/types/:id/color", payload, {
      pathParams: { id },
      schema: parseNull,
    });
  },

  deleteType(id: number): Promise<null> {
    return api.delete<null>("/api/v1/config/categories/types/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },

  createCategory(payload: CreateCategoryPayload): Promise<CategoryItem> {
    return api.post<CategoryItem>("/api/v1/config/categories", payload, {
      schema: parseCategoryItem,
    });
  },

  updateCategory(id: number, payload: UpdateNamePayload): Promise<CategoryItem> {
    return api.patch<CategoryItem>("/api/v1/config/categories/:id", payload, {
      pathParams: { id },
      schema: parseCategoryItem,
    });
  },

  deleteCategory(id: number): Promise<null> {
    return api.delete<null>("/api/v1/config/categories/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },

  createSubcategory(payload: CreateSubcategoryPayload): Promise<SubcategoryItem> {
    return api.post<SubcategoryItem>("/api/v1/config/categories/subcategories", payload, {
      schema: parseSubcategoryItem,
    });
  },

  updateSubcategory(id: number, payload: UpdateNamePayload): Promise<SubcategoryItem> {
    return api.patch<SubcategoryItem>("/api/v1/config/categories/subcategories/:id", payload, {
      pathParams: { id },
      schema: parseSubcategoryItem,
    });
  },

  deleteSubcategory(id: number): Promise<null> {
    return api.delete<null>("/api/v1/config/categories/subcategories/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },

  resetDefaults(): Promise<null> {
    return api.post<null>("/api/v1/config/hierarchy/reset", undefined, {
      schema: parseNull,
    });
  },
};
