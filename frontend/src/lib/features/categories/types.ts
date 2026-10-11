/**
 * Category domain models, palette colors, and runtime schema decoders.
 *
 * Implements "Parse, Don't Validate" decoders for category taxonomy responses
 * and preset color palettes across the backend API boundary.
 */

import { ContractViolationError, isObject } from "$lib/api/contracts";
import { CategoryId, ColorId, SubcategoryId, TypeId } from "$lib/types";

export { CategoryId, ColorId, SubcategoryId, TypeId };

export interface SubcategoryItem {
  readonly id: SubcategoryId;
  name: string;
}

export interface CategoryItem {
  readonly id: CategoryId;
  name: string;
  subcategories: SubcategoryItem[];
}

export interface TransactionTypeItem {
  readonly id: TypeId;
  readonly name: string;
  color: string;
  colorId: ColorId;
  categories: CategoryItem[];
}

export interface Color {
  readonly id: ColorId;
  readonly name: string;
  readonly hex: string;
}

/**
 * Compares two colors for identity equality based on domain ColorId.
 */
export function isSameColor(a: Color, b: Color): boolean {
  return a.id === b.id;
}

export interface CategoryHierarchyResponse {
  readonly types: TransactionTypeItem[];
  readonly colors: Color[];
}

/**
 * Request payload to create a new transaction type.
 */
export interface CreateTypePayload {
  readonly name: string;
  readonly colorId: ColorId;
}

/**
 * Request payload to update the display color of a transaction type.
 */
export interface UpdateTypeColorPayload {
  readonly colorId: ColorId;
}

/**
 * Request payload to create a new category under a transaction type.
 */
export interface CreateCategoryPayload {
  readonly typeId: TypeId;
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
  readonly categoryId: CategoryId;
  readonly name: string;
}

/**
 * UI & Chart specific presentation types
 */
export type TransactionType = string;

export interface PresentationCategoryItem {
  readonly id: CategoryId;
  readonly name: string;
  readonly type: string;
  readonly typeId: TypeId;
  readonly subcategories: readonly SubcategoryItem[];
}

export type SelectedCategoryNode =
  | {
      readonly kind: "type";
      readonly id: TypeId;
      readonly type: TransactionType;
      name: string;
      readonly parentName: null;
      readonly categoryId: null;
    }
  | {
      readonly kind: "category";
      readonly id: CategoryId;
      readonly type: TransactionType;
      name: string;
      readonly parentName: string;
      readonly categoryId: null;
    }
  | {
      readonly kind: "subcategory";
      readonly id: SubcategoryId;
      readonly type: TransactionType;
      name: string;
      readonly parentName: string;
      readonly categoryId: CategoryId;
    };

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
 * Validates and narrows raw JSON data to a strongly-typed Color.
 */
export function parseColor(raw: unknown): Color {
  if (isObject(raw) === false) {
    throw new ContractViolationError("Color payload must be an object", raw);
  }
  const id = ColorId(raw.id);
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("Color.name must be a string", raw);
  }
  if (typeof raw.hex !== "string") {
    throw new ContractViolationError("Color.hex must be a string", raw);
  }
  return Object.freeze({
    id,
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
  const id = SubcategoryId(raw.id);
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("SubcategoryItem.name must be a string", raw);
  }
  return Object.freeze({
    id,
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
  const id = CategoryId(raw.id);
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("CategoryItem.name must be a string", raw);
  }
  if (Array.isArray(raw.subcategories) === false) {
    throw new ContractViolationError("CategoryItem.subcategories must be an array", raw);
  }
  return {
    id,
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
  const id = TypeId(raw.id);
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("TransactionTypeItem.name must be a string", raw);
  }
  if (typeof raw.color !== "string") {
    throw new ContractViolationError("TransactionTypeItem.color must be a string", raw);
  }
  const colorId = ColorId(raw.colorId);
  const categories = Array.isArray(raw.categories) ? raw.categories.map(parseCategoryItem) : [];
  return {
    id,
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
  if (Array.isArray(raw.colors) === false) {
    throw new ContractViolationError("CategoryHierarchyResponse.colors must be an array", raw);
  }
  return {
    types: raw.types.map(parseTransactionTypeItem),
    colors: raw.colors.map(parseColor),
  };
}
