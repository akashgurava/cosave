import { apiFetch, type ApiResponse } from "$lib/api";
import {
  parseCategoryHierarchyResponse,
  parseCategoryItem,
  parseSubcategoryItem,
  parseTransactionTypeItem,
  type CategoryHierarchyResponse,
  type CategoryItem,
  type CreateCategoryPayload,
  type CreateSubcategoryPayload,
  type CreateTypePayload,
  type SubcategoryItem,
  type TransactionTypeItem,
  type UpdateNamePayload,
  type UpdateTypeColorPayload,
} from "./types";

/**
 * Category & Transaction Hierarchy API service functions.
 * The Rust backend is the authoritative Single Source of Truth (SSOT).
 */
export const categoriesApi = {
  async getHierarchy(): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      "/api/v1/categories",
      {},
      parseCategoryHierarchyResponse,
    );
  },

  async createType(payload: CreateTypePayload): Promise<ApiResponse<TransactionTypeItem>> {
    return apiFetch<TransactionTypeItem>(
      "/api/v1/categories/types",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseTransactionTypeItem,
    );
  },

  async updateTypeColor(
    id: string,
    color: string,
  ): Promise<ApiResponse<CategoryHierarchyResponse>> {
    const payload: UpdateTypeColorPayload = { color };
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/types/${encodeURIComponent(id)}/color`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseCategoryHierarchyResponse,
    );
  },

  async deleteType(id: string): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/types/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseCategoryHierarchyResponse,
    );
  },

  async createCategory(payload: CreateCategoryPayload): Promise<ApiResponse<CategoryItem>> {
    return apiFetch<CategoryItem>(
      "/api/v1/categories",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseCategoryItem,
    );
  },

  async updateCategory(id: string, name: string): Promise<ApiResponse<CategoryHierarchyResponse>> {
    const payload: UpdateNamePayload = { name };
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/${encodeURIComponent(id)}`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseCategoryHierarchyResponse,
    );
  },

  async deleteCategory(id: string): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseCategoryHierarchyResponse,
    );
  },

  async createSubcategory(
    payload: CreateSubcategoryPayload,
  ): Promise<ApiResponse<SubcategoryItem>> {
    return apiFetch<SubcategoryItem>(
      "/api/v1/categories/subcategories",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseSubcategoryItem,
    );
  },

  async updateSubcategory(
    id: string,
    name: string,
  ): Promise<ApiResponse<CategoryHierarchyResponse>> {
    const payload: UpdateNamePayload = { name };
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/subcategories/${encodeURIComponent(id)}`,
      {
        method: "PATCH",
        body: JSON.stringify(payload),
      },
      parseCategoryHierarchyResponse,
    );
  },

  async deleteSubcategory(id: string): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      `/api/v1/categories/subcategories/${encodeURIComponent(id)}`,
      {
        method: "DELETE",
      },
      parseCategoryHierarchyResponse,
    );
  },

  async resetDefaults(): Promise<ApiResponse<CategoryHierarchyResponse>> {
    return apiFetch<CategoryHierarchyResponse>(
      "/api/v1/categories/reset",
      {
        method: "POST",
      },
      parseCategoryHierarchyResponse,
    );
  },
};
