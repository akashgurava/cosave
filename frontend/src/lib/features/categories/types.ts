import { ContractViolationError, isObject } from "$lib/api";

export interface TransactionTypeItem {
  id: string;
  name: string;
  color: string;
  color_id?: number;
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

export interface ColorOption {
  id: number;
  name: string;
  hex: string;
}

export const PRESET_COLORS: readonly ColorOption[] = [
  { id: 1, name: "Emerald", hex: "#10b981" },
  { id: 2, name: "Rose", hex: "#f43f5e" },
  { id: 3, name: "Grey", hex: "#71717a" },
  { id: 4, name: "Blue", hex: "#3b82f6" },
  { id: 5, name: "Amber", hex: "#f59e0b" },
  { id: 6, name: "Violet", hex: "#8b5cf6" },
  { id: 7, name: "Cyan", hex: "#06b6d4" },
  { id: 8, name: "Orange", hex: "#f97316" },
  { id: 9, name: "Pink", hex: "#ec4899" },
  { id: 10, name: "Teal", hex: "#14b8a6" },
  { id: 11, name: "Indigo", hex: "#6366f1" },
  { id: 12, name: "Lime", hex: "#84cc16" },
] as const;

export interface CategoryHierarchyResponse {
  types: TransactionTypeItem[];
  categories: CategoryItem[];
  colors: ColorOption[];
}

/**
 * Request payload to create a new transaction type.
 */
export interface CreateTypePayload {
  name: string;
  color: string;
  color_id?: number;
}

/**
 * Request payload to update the display color of a transaction type.
 */
export interface UpdateTypeColorPayload {
  color: string;
  color_id?: number;
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

export interface SelectedCategoryNode {
  id: string;
  type: TransactionType;
  kind: "type" | "category" | "subcategory";
  name: string;
  parentName: string | null;
  categoryId: string | null;
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
 * Validates and narrows raw JSON data to a strongly-typed ColorOption.
 */
export function parseColorOption(raw: unknown): ColorOption {
  if (!isObject(raw)) {
    throw new ContractViolationError("ColorOption payload must be an object", raw);
  }
  if (typeof raw.id !== "number") {
    throw new ContractViolationError("ColorOption.id must be a number", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("ColorOption.name must be a string", raw);
  }
  if (typeof raw.hex !== "string") {
    throw new ContractViolationError("ColorOption.hex must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    hex: raw.hex,
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
  const color_id = typeof raw.color_id === "number" ? raw.color_id : undefined;
  return {
    id: raw.id,
    name: raw.name,
    color: raw.color,
    ...(color_id !== undefined ? { color_id } : {}),
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
  const colors = Array.isArray(raw.colors) ? raw.colors.map(parseColorOption) : [...PRESET_COLORS];
  return {
    types: raw.types.map(parseTransactionTypeItem),
    categories: raw.categories.map(parseCategoryItem),
    colors,
  };
}
