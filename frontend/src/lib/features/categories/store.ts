import { expectPresent } from "$lib/types";
import { categoriesApi } from "./api";
import { getTypeColor, isColorUsed, projectSankeyGraph } from "./sankey";
import {
  PRESET_COLORS,
  type CategoryItem,
  type ColorOption,
  type PresentationCategoryItem,
  type SankeyLinkData,
  type SankeyNodeData,
  type SelectedCategoryNode,
  type SubcategoryItem,
  type TransactionType,
  type TransactionTypeItem,
} from "./types";

export class CategoryStore {
  // Pure presentation-layer mirror of the Rust backend SSOT
  private typesState = $state<TransactionTypeItem[]>([]);
  private colorsState = $state<ColorOption[]>([...PRESET_COLORS]);
  private selectedNodeState = $state<SelectedCategoryNode | null>(null);
  private versionState = $state<number>(0);
  private isLoadingState = $state<boolean>(false);
  private isLoadedState = $state<boolean>(false);
  private errorState = $state<string | null>(null);

  public get colors(): ColorOption[] {
    return this.colorsState;
  }

  public get version(): number {
    return this.versionState;
  }

  public get isLoading(): boolean {
    return this.isLoadingState;
  }

  public get isLoaded(): boolean {
    return this.isLoadedState;
  }

  public get error(): string | null {
    return this.errorState;
  }

  private notify(): void {
    this.versionState++;
  }

  private categoryMap = $derived.by(() => {
    const map = new Map<number, PresentationCategoryItem>();
    for (const t of this.typesState) {
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

  private colorByHexMap = $derived(new Map(this.colorsState.map((c) => [c.hex.toLowerCase(), c])));

  private colorByIdMap = $derived(new Map(this.colorsState.map((c) => [c.id, c])));

  private typeByNameMap = $derived(new Map(this.typesState.map((t) => [t.name.toLowerCase(), t])));

  private typeByIdMap = $derived(new Map(this.typesState.map((t) => [t.id, t])));

  public get types(): TransactionTypeItem[] {
    return this.typesState;
  }

  public get categories(): PresentationCategoryItem[] {
    return Array.from(this.categoryMap.values());
  }

  public getCategory(id: number): PresentationCategoryItem {
    return expectPresent(
      this.categoryMap.get(id),
      "STORE.CATEGORY.GET_CATEGORY_BY_ID",
      `Category ${id} not found in category store`,
    );
  }

  public getColor(id: number): ColorOption {
    return expectPresent(
      this.colorByIdMap.get(id),
      "STORE.CATEGORY.GET_COLOR_BY_ID",
      `Color ${id} not found in category store`,
    );
  }

  public getColorByHex(hex: string): ColorOption {
    return expectPresent(
      this.colorByHexMap.get(hex.toLowerCase()),
      "STORE.CATEGORY.GET_COLOR_BY_HEX",
      `Color hex "${hex}" not found in category palette`,
    );
  }

  public getTypeById(id: number): TransactionTypeItem {
    return expectPresent(
      this.typeByIdMap.get(id),
      "STORE.CATEGORY.GET_TYPE_BY_ID",
      `Type ${id} not found in category store`,
    );
  }

  public get selectedNode(): SelectedCategoryNode | null {
    return this.selectedNodeState;
  }

  public setSelectedNode(node: SelectedCategoryNode | null): void {
    this.selectedNodeState = node;
  }

  public getType(name: string): TransactionTypeItem | null {
    return this.typeByNameMap.get(name.toLowerCase()) ?? null;
  }

  public getTypeColor(typeName: string): { solid: string; subtle: string; border: string } {
    return getTypeColor(this.typesState, typeName);
  }

  public isColorUsed(hex: string, excludeTypeName?: string): boolean {
    return isColorUsed(this.typesState, hex, excludeTypeName);
  }

  /**
   * Loads the authoritative hierarchy from the Rust backend SSOT.
   */
  public async load(): Promise<void> {
    this.isLoadingState = true;
    this.errorState = null;
    try {
      const res = await categoriesApi.getHierarchy();
      this.typesState = res.types;
      if (res.colors && res.colors.length > 0) {
        this.colorsState = res.colors;
      }
      this.isLoadedState = true;
      this.notify();
      console.info(
        `[cosave:categories] Loaded hierarchy: ${this.typesState.length} types, ${this.categories.length} categories`,
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to load categories";
      this.errorState = msg;
      console.error("[cosave:categories] Hierarchy load failed:", err);
    } finally {
      this.isLoadingState = false;
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
        : (this.colorByHexMap.get(colorOrColorId.toLowerCase())?.id ?? 1);

    try {
      const res = await categoriesApi.createType({ name: trimmed, colorId });
      this.typesState.push(res);
      this.notify();
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
        ? this.typeByIdMap.get(typeNameOrId)
        : this.typeByNameMap.get(typeNameOrId.toLowerCase());
    if (!found) return false;

    const colorId =
      typeof newColorOrColorId === "number"
        ? newColorOrColorId
        : (this.colorByHexMap.get(newColorOrColorId.toLowerCase())?.id ?? found.colorId);

    try {
      await categoriesApi.updateTypeColor(found.id, { colorId });
      found.colorId = colorId;
      const colorObj = this.colorByIdMap.get(colorId);
      if (colorObj) {
        found.color = colorObj.hex;
      }
      this.notify();
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
    const target = this.typesState.find((t) =>
      typeof typeNameOrId === "number"
        ? t.id === typeNameOrId
        : t.name.toLowerCase() === typeNameOrId.toLowerCase(),
    );
    if (!target) return false;

    try {
      await categoriesApi.deleteType(target.id);
      this.typesState = this.typesState.filter((t) => t.id !== target.id);
      if (
        this.selectedNodeState?.type.toLowerCase() === target.name.toLowerCase() ||
        this.selectedNodeState?.id === target.id
      ) {
        this.selectedNodeState = null;
      }
      this.notify();
      console.info(`[cosave:categories] Deleted type: ${target.name} (${target.id})`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Delete type failed:", err);
      return false;
    }
  }

  /**
   * Creates a new mid-level category under a transaction type.
   */
  public async addCategory(
    type: TransactionType | number,
    name: string,
  ): Promise<CategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    const foundType = this.typesState.find((t) =>
      typeof type === "number"
        ? t.id === type
        : t.name.toLowerCase() === String(type).toLowerCase() || String(t.id) === String(type),
    );
    const typeId = foundType !== undefined ? foundType.id : Number(type);

    try {
      const res = await categoriesApi.createCategory({ typeId, name: trimmed });
      if (foundType) {
        foundType.categories.push(res);
      }
      this.notify();
      console.info(
        `[cosave:categories] Added category: ${res.name} under ${foundType?.name ?? type}`,
      );
      return res;
    } catch (err) {
      console.error("[cosave:categories] Add category failed:", err);
      throw err;
    }
  }

  /**
   * Creates a new leaf subcategory under a category.
   */
  public async addSubcategory(
    categoryId: number | string,
    name: string,
  ): Promise<SubcategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;
    const catIdNum = Number(categoryId);

    try {
      const res = await categoriesApi.createSubcategory({ categoryId: catIdNum, name: trimmed });
      for (const t of this.typesState) {
        const cat = t.categories.find((c) => c.id === catIdNum);
        if (cat) {
          cat.subcategories.push(res);
          break;
        }
      }
      this.notify();
      console.info(`[cosave:categories] Added subcategory: ${res.name} to category ${categoryId}`);
      return res;
    } catch (err) {
      console.error("[cosave:categories] Add subcategory failed:", err);
      throw err;
    }
  }

  /**
   * Renames a category.
   */
  public async renameCategory(categoryId: number | string, newName: string): Promise<boolean> {
    const trimmed = newName.trim();
    if (!trimmed) return false;
    const catIdNum = Number(categoryId);

    try {
      const res = await categoriesApi.updateCategory(catIdNum, { name: trimmed });
      for (const t of this.typesState) {
        const cat = t.categories.find((c) => c.id === catIdNum);
        if (cat) {
          cat.name = res.name;
          break;
        }
      }
      if (this.selectedNodeState?.id === catIdNum) {
        this.selectedNodeState.name = trimmed;
      }
      this.notify();
      console.info(`[cosave:categories] Renamed category ${categoryId} -> ${trimmed}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Rename category failed:", err);
      throw err;
    }
  }

  /**
   * Renames a leaf subcategory.
   * Can be called as renameSubcategory(subcategoryId, newName)
   * or as renameSubcategory(categoryId, subcategoryId, newName).
   */
  public async renameSubcategory(
    subcategoryIdOrCategoryId: number | string,
    newNameOrSubcategoryId: number | string,
    optionalNewName?: string,
  ): Promise<boolean> {
    const subIdNum = Number(
      optionalNewName !== undefined ? newNameOrSubcategoryId : subcategoryIdOrCategoryId,
    );
    const newName = (optionalNewName ?? String(newNameOrSubcategoryId)).trim();
    if (!newName) return false;

    try {
      const res = await categoriesApi.updateSubcategory(subIdNum, { name: newName });
      for (const t of this.typesState) {
        for (const c of t.categories) {
          const sub = c.subcategories.find((s) => s.id === subIdNum);
          if (sub) {
            sub.name = res.name;
            break;
          }
        }
      }
      if (this.selectedNodeState?.id === subIdNum) {
        this.selectedNodeState.name = newName;
      }
      this.notify();
      console.info(`[cosave:categories] Renamed subcategory ${subIdNum} -> ${newName}`);
      return true;
    } catch (err) {
      console.error("[cosave:categories] Rename subcategory failed:", err);
      throw err;
    }
  }

  /**
   * Deletes a category and cascades to its subcategories.
   */
  public async deleteCategory(categoryId: number | string): Promise<boolean> {
    const catIdNum = Number(categoryId);

    try {
      await categoriesApi.deleteCategory(catIdNum);
      for (const t of this.typesState) {
        t.categories = t.categories.filter((c) => c.id !== catIdNum);
      }
      if (
        this.selectedNodeState?.id === catIdNum ||
        this.selectedNodeState?.categoryId === catIdNum
      ) {
        this.selectedNodeState = null;
      }
      this.notify();
      console.info(`[cosave:categories] Deleted category: ${categoryId}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to delete category";
      console.error("[cosave:categories] Delete category failed:", err);
      return false;
    }
  }

  /**
   * Deletes a leaf subcategory.
   * Can be called as deleteSubcategory(subcategoryId)
   * or as deleteSubcategory(categoryId, subcategoryId).
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
      for (const t of this.typesState) {
        for (const c of t.categories) {
          c.subcategories = c.subcategories.filter((s) => s.id !== subIdNum);
        }
      }
      if (this.selectedNodeState?.id === subIdNum) {
        this.selectedNodeState = null;
      }
      this.notify();
      console.info(`[cosave:categories] Deleted subcategory: ${subIdNum}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to delete subcategory";
      console.error("[cosave:categories] Delete subcategory failed:", err);
      return false;
    }
  }

  /**
   * Atomically resets categories back to authoritative backend defaults.
   */
  public async resetDefaults(): Promise<boolean> {
    this.isLoadingState = true;
    try {
      await categoriesApi.resetDefaults();
      await this.load();
      this.selectedNodeState = null;
      console.info("[cosave:categories] Reset categories back to authoritative defaults");
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to reset defaults";
      console.error("[cosave:categories] Reset defaults failed:", err);
      return false;
    } finally {
      this.isLoadingState = false;
    }
  }

  public getSankeyData(activeFilter: string | string[] = "All"): {
    nodes: SankeyNodeData[];
    links: SankeyLinkData[];
  } {
    return projectSankeyGraph(this.typesState, this.categories, activeFilter);
  }
}

export const categoryStore = new CategoryStore();
