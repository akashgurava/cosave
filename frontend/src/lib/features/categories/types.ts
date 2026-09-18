import { ContractViolationError, isObject } from "$lib/api";

export interface TransactionTypeItem {
  id: string;
  name: string;
  color: string;
}

export interface SubcategoryItem {
  id: string;
  name: string;
}

export interface CategoryItem {
  id: string;
  name: string;
  type: string;
  subcategories: SubcategoryItem[];
}

export interface CategoryHierarchyResponse {
  types: TransactionTypeItem[];
  categories: CategoryItem[];
}

/**
 * Request payload to create a new transaction type.
 */
export interface CreateTypePayload {
  name: string;
  color: string;
}

/**
 * Request payload to update the display color of a transaction type.
 */
export interface UpdateTypeColorPayload {
  color: string;
}

/**
 * Request payload to create a new category under a transaction type.
 */
export interface CreateCategoryPayload {
  type_name: string;
  name: string;
}

/**
 * Generic request payload to rename an entity (category or subcategory).
 */
export interface UpdateNamePayload {
  name: string;
}

/**
 * Request payload to create a new subcategory under an existing category.
 */
export interface CreateSubcategoryPayload {
  category_id: string;
  name: string;
}

/**
 * UI & Chart specific types
 */
export type TransactionType = string;

export interface ColorOption {
  id: string;
  name: string;
  hex: string;
}

export const PRESET_COLORS: readonly ColorOption[] = [
  { id: "emerald", name: "Emerald", hex: "#10b981" },
  { id: "rose", name: "Rose", hex: "#f43f5e" },
  { id: "zinc", name: "Grey", hex: "#71717a" },
  { id: "blue", name: "Blue", hex: "#3b82f6" },
  { id: "amber", name: "Amber", hex: "#f59e0b" },
  { id: "violet", name: "Violet", hex: "#8b5cf6" },
  { id: "cyan", name: "Cyan", hex: "#06b6d4" },
  { id: "orange", name: "Orange", hex: "#f97316" },
  { id: "pink", name: "Pink", hex: "#ec4899" },
  { id: "teal", name: "Teal", hex: "#14b8a6" },
  { id: "indigo", name: "Indigo", hex: "#6366f1" },
  { id: "lime", name: "Lime", hex: "#84cc16" },
] as const;

export interface SelectedCategoryNode {
  id: string;
  type: TransactionType;
  kind: "type" | "category" | "subcategory";
  name: string;
  parentName?: string;
  categoryId?: string;
}

export interface SankeyNodeData {
  name: string;
  displayName: string;
  depth: number;
  level: "type" | "category" | "subcategory";
  type: TransactionType;
  categoryName?: string;
  value?: number;
  entity: SelectedCategoryNode;
  itemStyle?: {
    color?: string;
    shadowBlur?: number;
    shadowColor?: string;
  };
}

export interface SankeyLinkData {
  source: string;
  target: string;
  value: number;
  lineStyle?: {
    color?: string;
    opacity?: number;
    shadowBlur?: number;
    shadowColor?: string;
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed TransactionTypeItem.
 */
export function parseTransactionTypeItem(raw: unknown): TransactionTypeItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("TransactionTypeItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("TransactionTypeItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("TransactionTypeItem.name must be a string", raw);
  }
  if (typeof raw.color !== "string") {
    throw new ContractViolationError("TransactionTypeItem.color must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    color: raw.color,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed SubcategoryItem.
 */
export function parseSubcategoryItem(raw: unknown): SubcategoryItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("SubcategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("SubcategoryItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("SubcategoryItem.name must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryItem.
 */
export function parseCategoryItem(raw: unknown): CategoryItem {
  if (!isObject(raw)) {
    throw new ContractViolationError("CategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("CategoryItem.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("CategoryItem.name must be a string", raw);
  }
  if (typeof raw.type !== "string") {
    throw new ContractViolationError("CategoryItem.type must be a string", raw);
  }
  if (!Array.isArray(raw.subcategories)) {
    throw new ContractViolationError("CategoryItem.subcategories must be an array", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    type: raw.type,
    subcategories: raw.subcategories.map(parseSubcategoryItem),
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryHierarchyResponse.
 */
export function parseCategoryHierarchyResponse(raw: unknown): CategoryHierarchyResponse {
  if (!isObject(raw)) {
    throw new ContractViolationError("CategoryHierarchyResponse payload must be an object", raw);
  }
  if (!Array.isArray(raw.types)) {
    throw new ContractViolationError("CategoryHierarchyResponse.types must be an array", raw);
  }
  if (!Array.isArray(raw.categories)) {
    throw new ContractViolationError("CategoryHierarchyResponse.categories must be an array", raw);
  }
  return {
    types: raw.types.map(parseTransactionTypeItem),
    categories: raw.categories.map(parseCategoryItem),
  };
}
