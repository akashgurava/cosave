/**
 * Category domain models, palette colors, and runtime schema decoders.
 *
 * Implements "Parse, Don't Validate" decoders for category taxonomy responses
 * and preset color palettes across the backend API boundary.
 */

import { ContractViolationError, isObject } from "$lib/api/contracts";

export interface SubcategoryItem {
  readonly id: number;
  name: string;
}

export interface CategoryItem {
  readonly id: number;
  name: string;
  subcategories: SubcategoryItem[];
}

export interface TransactionTypeItem {
  readonly id: number;
  readonly name: string;
  color: string;
  colorId: number;
  categories: CategoryItem[];
}

export interface ColorOption {
  readonly id: number;
  readonly name: string;
  readonly hex: string;
}

export const PRESET_COLORS: readonly ColorOption[] = Object.freeze([
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
]);

export interface CategoryHierarchyResponse {
  readonly types: TransactionTypeItem[];
  readonly colors: ColorOption[];
}

/**
 * Request payload to create a new transaction type.
 */
export interface CreateTypePayload {
  readonly name: string;
  readonly colorId: number;
}

/**
 * Request payload to update the display color of a transaction type.
 */
export interface UpdateTypeColorPayload {
  readonly colorId: number;
}

/**
 * Request payload to create a new category under a transaction type.
 */
export interface CreateCategoryPayload {
  readonly typeId: number;
  readonly name: string;
}

/**
 * Generic request payload to rename an entity (category or subcategory).
 */
export interface UpdateNamePayload {
  readonly name: string;
}

/**
 * Request payload to create a new subcategory under an existing category.
 */
export interface CreateSubcategoryPayload {
  readonly categoryId: number;
  readonly name: string;
}

/**
 * UI & Chart specific presentation types
 */
export type TransactionType = string;

export interface PresentationCategoryItem {
  readonly id: number;
  readonly name: string;
  readonly type: string;
  readonly typeId: number;
  readonly subcategories: readonly SubcategoryItem[];
}

export interface SelectedCategoryNode {
  readonly id: number;
  readonly type: TransactionType;
  readonly kind: "type" | "category" | "subcategory";
  name: string;
  readonly parentName: string | null;
  readonly categoryId: number | null;
}

export interface SankeyNodeData {
  readonly name: string;
  readonly displayName: string;
  readonly depth: number;
  readonly level: "type" | "category" | "subcategory";
  readonly type: TransactionType;
  readonly categoryName?: string;
  readonly value?: number;
  readonly entity: SelectedCategoryNode;
  readonly itemStyle?: {
    readonly color?: string;
    readonly shadowBlur?: number;
    readonly shadowColor?: string;
  };
}

export interface SankeyLinkData {
  readonly source: string;
  readonly target: string;
  readonly value: number;
  readonly lineStyle?: {
    readonly color?: string;
    readonly opacity?: number;
    readonly shadowBlur?: number;
    readonly shadowColor?: string;
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed ColorOption.
 */
export function parseColorOption(raw: unknown): ColorOption {
  if (isObject(raw) === false) {
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
  return Object.freeze({
    id: raw.id,
    name: raw.name,
    hex: raw.hex,
  });
}

/**
 * Validates and narrows raw JSON data to a strongly-typed SubcategoryItem.
 */
export function parseSubcategoryItem(raw: unknown): SubcategoryItem {
  if (isObject(raw) === false) {
    throw new ContractViolationError("SubcategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "number") {
    throw new ContractViolationError("SubcategoryItem.id must be a number", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("SubcategoryItem.name must be a string", raw);
  }
  return Object.freeze({
    id: raw.id,
    name: raw.name,
  });
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryItem.
 */
export function parseCategoryItem(raw: unknown): CategoryItem {
  if (isObject(raw) === false) {
    throw new ContractViolationError("CategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "number") {
    throw new ContractViolationError("CategoryItem.id must be a number", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("CategoryItem.name must be a string", raw);
  }
  if (Array.isArray(raw.subcategories) === false) {
    throw new ContractViolationError("CategoryItem.subcategories must be an array", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    subcategories: raw.subcategories.map(parseSubcategoryItem),
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed TransactionTypeItem.
 */
export function parseTransactionTypeItem(raw: unknown): TransactionTypeItem {
  if (isObject(raw) === false) {
    throw new ContractViolationError("TransactionTypeItem payload must be an object", raw);
  }
  if (typeof raw.id !== "number") {
    throw new ContractViolationError("TransactionTypeItem.id must be a number", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("TransactionTypeItem.name must be a string", raw);
  }
  if (typeof raw.color !== "string") {
    throw new ContractViolationError("TransactionTypeItem.color must be a string", raw);
  }
  const colorId = typeof raw.colorId === "number" ? raw.colorId : 0;
  const categories = Array.isArray(raw.categories) ? raw.categories.map(parseCategoryItem) : [];
  return {
    id: raw.id,
    name: raw.name,
    color: raw.color,
    colorId,
    categories,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CategoryHierarchyResponse.
 */
export function parseCategoryHierarchyResponse(raw: unknown): CategoryHierarchyResponse {
  if (isObject(raw) === false) {
    throw new ContractViolationError("CategoryHierarchyResponse payload must be an object", raw);
  }
  if (Array.isArray(raw.types) === false) {
    throw new ContractViolationError("CategoryHierarchyResponse.types must be an array", raw);
  }
  const colors = Array.isArray(raw.colors) ? raw.colors.map(parseColorOption) : [...PRESET_COLORS];
  return {
    types: raw.types.map(parseTransactionTypeItem),
    colors,
  };
}
