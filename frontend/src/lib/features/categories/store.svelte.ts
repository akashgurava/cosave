/**
 * Reactive store managing category hierarchy, color palettes, and graph projections.
 *
 * Serves as the authoritative client-side presentation mirror for the backend
 * category taxonomy subsystem. Enforces immutable state re-assignment on all writes
 * so that all reactive derived lookup maps ($derived) invalidate cleanly and instantly.
 */

import { SvelteMap } from "svelte/reactivity";
import { expectPresent, type AsyncState } from "$lib/types";
import { categoriesApi } from "./api";
import { getTypeColor, isColorUsed, projectSankeyGraph } from "./sankey";
import {
  PRESET_COLORS,
  type CategoryHierarchyResponse,
  type CategoryItem,
  type ColorOption,
  type PresentationCategoryItem,
  type SankeyLinkData,
  type SankeyNodeData,
  type SelectedCategoryNode,
  type SubcategoryItem,
  type TransactionTypeItem,
} from "./types";

export class CategoryStore {
  // Pure presentation-layer mirror of the Rust backend SSOT via AsyncState
  #state = $state<AsyncState<CategoryHierarchyResponse>>({ status: "idle" });
  #typesState = $state<TransactionTypeItem[]>([]);
  #colorsState = $state<ColorOption[]>([...PRESET_COLORS]);
  #selectedNodeState = $state<SelectedCategoryNode | null>(null);

  public get state(): AsyncState<CategoryHierarchyResponse> {
    return this.#state;
  }

  public get colors(): readonly ColorOption[] {
    return this.#colorsState;
  }

  public get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  public get isLoaded(): boolean {
    return this.#state.status === "success";
  }

  public get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  #categoryMap = $derived.by(() => {
    const map = new SvelteMap<number, PresentationCategoryItem>();
    for (const t of this.#typesState) {
      for (const c of t.categories) {
        map.set(c.id, {
          id: c.id,
          name: c.name,
          type: t.name,
          typeId: t.id,
          subcategories: c.subcategories,
        });
      }
    }
    return map;
  });

  #colorByHexMap = $derived(new SvelteMap(this.#colorsState.map((c) => [c.hex.toLowerCase(), c])));

  #colorByIdMap = $derived(new SvelteMap(this.#colorsState.map((c) => [c.id, c])));

  #typeByNameMap = $derived(new SvelteMap(this.#typesState.map((t) => [t.name.toLowerCase(), t])));

  #typeByIdMap = $derived(new SvelteMap(this.#typesState.map((t) => [t.id, t])));

  public get types(): readonly TransactionTypeItem[] {
    return this.#typesState;
  }

  public get categories(): readonly PresentationCategoryItem[] {
    return Array.from(this.#categoryMap.values());
  }

  public getCategory(id: number): PresentationCategoryItem {
    return expectPresent(
      this.#categoryMap.get(id),
      "STORE.CATEGORY.GET_CATEGORY_BY_ID",
      `Category ${id} not found in category store`,
    );
  }

  public getColor(id: number): ColorOption {
    return expectPresent(
      this.#colorByIdMap.get(id),
      "STORE.CATEGORY.GET_COLOR_BY_ID",
      `Color ${id} not found in category store`,
    );
  }

  public getColorByHex(hex: string): ColorOption {
    return expectPresent(
      this.#colorByHexMap.get(hex.toLowerCase()),
      "STORE.CATEGORY.GET_COLOR_BY_HEX",
      `Color hex "${hex}" not found in category palette`,
    );
  }

  public getTypeById(id: number): TransactionTypeItem {
    return expectPresent(
      this.#typeByIdMap.get(id),
      "STORE.CATEGORY.GET_TYPE_BY_ID",
      `Type ${id} not found in category store`,
    );
  }

  public get selectedNode(): SelectedCategoryNode | null {
    return this.#selectedNodeState;
  }

  public setSelectedNode(node: SelectedCategoryNode | null): void {
    this.#selectedNodeState = node;
  }

  public getType(name: string): TransactionTypeItem | null {
    return this.#typeByNameMap.get(name.toLowerCase()) ?? null;
  }

  public getTypeColor(typeName: string): { solid: string; subtle: string; border: string } {
    return getTypeColor(this.#typesState, typeName);
  }

  public isColorUsed(hex: string, excludeTypeName?: string): boolean {
    return isColorUsed(this.#typesState, hex, excludeTypeName);
  }

  /**
   * Helper that updates #typesState by cloning and reassigning the root array,
   * guaranteeing that all $derived maps rebuild immediately.
   */
  #updateTypes(updater: (types: TransactionTypeItem[]) => void): void {
    const cloned = structuredClone(this.#typesState);
    updater(cloned);
    this.#typesState = cloned;
  }

  /**
   * Loads the authoritative hierarchy from the Rust backend SSOT.
   */
  public async load(): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const res = await categoriesApi.getHierarchy();
      this.#typesState = res.types;
      if (res.colors && res.colors.length > 0) {
        this.#colorsState = res.colors;
      }
      this.#state = { status: "success", data: res };
      console.info(
        `[cosave:categories] Loaded hierarchy: ${this.#typesState.length} types, ${this.categories.length} categories`,
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to load categories";
      this.#state = {
        status: "error",
        error: { action: "STORE.CATEGORIES.LOAD_FAILED", message: msg },
      };
      console.error("[cosave:categories] Hierarchy load failed:", err);
    }
  }

  /**
   * Creates a new root transaction type with an associated theme color.
   */
  public async addType(
    name: string,
    colorOrColorId: string | number,
  ): Promise<TransactionTypeItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    const colorId =
      typeof colorOrColorId === "number"
        ? colorOrColorId
        : (this.#colorByHexMap.get(colorOrColorId.toLowerCase())?.id ?? 1);

    try {
      const res = await categoriesApi.createType({ name: trimmed, colorId });
      this.#updateTypes((types) => {
        types.push(res);
      });
      console.info(`[cosave:categories] Added type: ${res.name} (${res.id})`);
      return res;
    } catch (err) {
      console.error("[cosave:categories] Create type failed:", err);
      throw err;
    }
  }

  /**
   * Updates the display color of an existing transaction type.
   */
  public async updateTypeColor(
    typeNameOrId: string | number,
    newColorOrColorId: string | number,
  ): Promise<boolean> {
    const found =
      typeof typeNameOrId === "number"
        ? this.#typeByIdMap.get(typeNameOrId)
        : this.#typeByNameMap.get(typeNameOrId.toLowerCase());
    if (!found) return false;

    const colorId =
      typeof newColorOrColorId === "number"
        ? newColorOrColorId
        : (this.#colorByHexMap.get(newColorOrColorId.toLowerCase())?.id ?? found.colorId);

    try {
      await categoriesApi.updateTypeColor(found.id, { colorId });
      const colorObj = this.#colorByIdMap.get(colorId);
      const newHex = colorObj ? colorObj.hex : found.color;

      this.#updateTypes((types) => {
        const target = types.find((t) => t.id === found.id);
        if (target) {
          target.colorId = colorId;
          target.color = newHex;
        }
      });
      console.info(`[cosave:categories] Updated type color: ${found.name} -> colorId ${colorId}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Update type color failed:", err);
      return false;
    }
  }

  /**
   * Deletes a transaction type and cascades deletion locally to mirrored child categories.
   */
  public async deleteType(typeNameOrId: string | number): Promise<boolean> {
    const target = this.#typesState.find((t) =>
      typeof typeNameOrId === "number"
        ? t.id === typeNameOrId
        : t.name.toLowerCase() === typeNameOrId.toLowerCase(),
    );
    if (!target) return false;

    try {
      await categoriesApi.deleteType(target.id);
      this.#updateTypes((types) => {
        const idx = types.findIndex((t) => t.id === target.id);
        if (idx !== -1) types.splice(idx, 1);
      });
      if (
        this.#selectedNodeState?.type.toLowerCase() === target.name.toLowerCase() ||
        this.#selectedNodeState?.id === target.id
      ) {
        this.#selectedNodeState = null;
      }
      console.info(`[cosave:categories] Deleted type: ${target.name} (${target.id})`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Delete type failed:", err);
      return false;
    }
  }

  /**
   * Renames an existing category.
   */
  public async renameCategory(categoryId: number, newName: string): Promise<boolean> {
    const trimmed = newName.trim();
    if (!trimmed) return false;

    try {
      await categoriesApi.updateCategory(categoryId, { name: trimmed });
      this.#updateTypes((types) => {
        for (const t of types) {
          const cat = t.categories.find((c) => c.id === categoryId);
          if (cat) {
            cat.name = trimmed;
            break;
          }
        }
      });
      if (this.#selectedNodeState?.id === categoryId) {
        this.#selectedNodeState.name = trimmed;
      }
      console.info(`[cosave:categories] Renamed category: ${categoryId} -> ${trimmed}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Rename category failed:", err);
      return false;
    }
  }

  /**
   * Renames a subcategory.
   */
  public async renameSubcategory(
    subcategoryIdOrCategoryId: number,
    newNameOrSubcategoryId: string | number,
    optionalNewName?: string,
  ): Promise<boolean> {
    let subId: number;
    let newName: string;

    if (typeof optionalNewName === "string") {
      subId = Number(newNameOrSubcategoryId);
      newName = optionalNewName.trim();
    } else {
      subId = subcategoryIdOrCategoryId;
      newName = String(newNameOrSubcategoryId).trim();
    }

    if (!newName) return false;

    try {
      await categoriesApi.updateSubcategory(subId, { name: newName });
      this.#updateTypes((types) => {
        for (const t of types) {
          for (const c of t.categories) {
            const sub = c.subcategories.find((s) => s.id === subId);
            if (sub) {
              sub.name = newName;
              break;
            }
          }
        }
      });
      if (this.#selectedNodeState?.id === subId) {
        this.#selectedNodeState.name = newName;
      }
      console.info(`[cosave:categories] Renamed subcategory: ${subId} -> ${newName}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Rename subcategory failed:", err);
      return false;
    }
  }

  /**
   * Creates a new child category under a transaction type.
   */
  public async addCategory(
    typeIdOrName: number | string,
    name: string,
  ): Promise<CategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    let targetType: TransactionTypeItem | undefined;
    if (typeof typeIdOrName === "number") {
      targetType = this.#typeByIdMap.get(typeIdOrName);
    } else {
      targetType = this.#typeByNameMap.get(typeIdOrName.toLowerCase());
    }

    if (!targetType) {
      console.error(`[cosave:categories] Type not found: ${typeIdOrName}`);
      return null;
    }

    try {
      const res = await categoriesApi.createCategory({ typeId: targetType.id, name: trimmed });
      this.#updateTypes((types) => {
        const found = types.find((t) => t.id === targetType.id);
        if (found) found.categories.push(res);
      });
      console.info(`[cosave:categories] Added category: ${res.name} to ${targetType.name}`);
      return res;
    } catch (err) {
      console.error("[cosave:categories] Add category failed:", err);
      throw err;
    }
  }

  /**
   * Creates a new child subcategory under an existing category.
   */
  public async addSubcategory(categoryId: number, name: string): Promise<SubcategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    try {
      const res = await categoriesApi.createSubcategory({ categoryId, name: trimmed });
      this.#updateTypes((types) => {
        for (const t of types) {
          const cat = t.categories.find((c) => c.id === categoryId);
          if (cat) {
            cat.subcategories.push(res);
            break;
          }
        }
      });
      console.info(`[cosave:categories] Added subcategory: ${res.name} to category ${categoryId}`);
      return res;
    } catch (err) {
      console.error("[cosave:categories] Add subcategory failed:", err);
      throw err;
    }
  }

  /**
   * Deletes a category and cascades deletion to child subcategories.
   */
  public async deleteCategory(categoryId: number): Promise<boolean> {
    try {
      await categoriesApi.deleteCategory(categoryId);
      this.#updateTypes((types) => {
        for (const t of types) {
          t.categories = t.categories.filter((c) => c.id !== categoryId);
        }
      });
      if (this.#selectedNodeState?.id === categoryId) {
        this.#selectedNodeState = null;
      }
      console.info(`[cosave:categories] Deleted category: ${categoryId}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Delete category failed:", err);
      return false;
    }
  }

  /**
   * Deletes a leaf subcategory.
   */
  public async deleteSubcategory(
    subcategoryIdOrCategoryId: number | string,
    optionalSubcategoryId?: number | string,
  ): Promise<boolean> {
    const subIdNum = Number(
      optionalSubcategoryId !== undefined ? optionalSubcategoryId : subcategoryIdOrCategoryId,
    );

    try {
      await categoriesApi.deleteSubcategory(subIdNum);
      this.#updateTypes((types) => {
        for (const t of types) {
          for (const c of t.categories) {
            c.subcategories = c.subcategories.filter((s) => s.id !== subIdNum);
          }
        }
      });
      if (this.#selectedNodeState?.id === subIdNum) {
        this.#selectedNodeState = null;
      }
      console.info(`[cosave:categories] Deleted subcategory: ${subIdNum}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Delete subcategory failed:", err);
      return false;
    }
  }

  /**
   * Atomically resets categories back to authoritative backend defaults.
   */
  public async resetDefaults(): Promise<boolean> {
    try {
      await categoriesApi.resetDefaults();
      await this.load();
      this.#selectedNodeState = null;
      console.info("[cosave:categories] Reset categories back to authoritative defaults");
      return true;
    } catch (err) {
      console.error("[cosave:categories] Reset defaults failed:", err);
      return false;
    }
  }

  public getSankeyData(activeFilter: string | string[] = "All"): {
    nodes: SankeyNodeData[];
    links: SankeyLinkData[];
  } {
    return projectSankeyGraph(this.#typesState, this.categories, activeFilter);
  }
}

export const categoryStore = new CategoryStore();
