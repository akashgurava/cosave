import { categoriesApi } from "./api";
import { getTypeColor, isColorUsed, projectSankeyGraph } from "./sankey";
import type {
  CategoryHierarchyResponse,
  CategoryItem,
  SankeyLinkData,
  SankeyNodeData,
  SelectedCategoryNode,
  SubcategoryItem,
  TransactionType,
  TransactionTypeItem,
} from "./types";

export class CategoryStore {
  // Pure presentation-layer mirror of the Rust backend SSOT
  private typesState = $state<TransactionTypeItem[]>([]);
  private categoriesState = $state<CategoryItem[]>([]);
  private selectedNodeState = $state<SelectedCategoryNode | null>(null);
  private versionState = $state<number>(0);
  private isLoadingState = $state<boolean>(false);
  private isLoadedState = $state<boolean>(false);
  private errorState = $state<string | null>(null);

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

  private notify() {
    this.versionState++;
  }

  public get types(): TransactionTypeItem[] {
    return this.typesState;
  }

  public get categories(): CategoryItem[] {
    return this.categoriesState;
  }

  public get selectedNode(): SelectedCategoryNode | null {
    return this.selectedNodeState;
  }

  public setSelectedNode(node: SelectedCategoryNode | null) {
    this.selectedNodeState = node;
  }

  public getType(name: string): TransactionTypeItem | undefined {
    return this.typesState.find((t) => t.name.toLowerCase() === name.toLowerCase());
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
      this.typesState = res.data.types;
      this.categoriesState = res.data.categories;
      this.isLoadedState = true;
      this.notify();
      console.info(
        `[cosave:categories] Loaded hierarchy: ${this.typesState.length} types, ${this.categoriesState.length} categories`,
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
  public async addType(name: string, color: string): Promise<TransactionTypeItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    try {
      const res = await categoriesApi.createType({ name: trimmed, color });
      this.typesState.push(res.data);
      this.notify();
      console.info(`[cosave:categories] Added type: ${res.data.name} (${res.data.id})`);
      return res.data;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to add type";
      console.error("[cosave:categories] Create type failed:", err);
    }

    return null;
  }

  private setHierarchy(data: CategoryHierarchyResponse) {
    this.typesState = data.types;
    this.categoriesState = data.categories;
    this.isLoadedState = true;
    this.notify();
  }

  /**
   * Updates the display color of an existing transaction type.
   */
  public async updateTypeColor(typeName: string, newColor: string): Promise<boolean> {
    const found = this.typesState.find((t) => t.name.toLowerCase() === typeName.toLowerCase());
    if (!found) return false;

    try {
      const res = await categoriesApi.updateTypeColor(found.id, newColor);
      this.setHierarchy(res.data);
      console.info(`[cosave:categories] Updated type color: ${typeName} -> ${newColor}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to update type color";
      console.error("[cosave:categories] Update type color failed:", err);
      return false;
    }
  }

  /**
   * Deletes a transaction type and cascades deletion locally to mirrored child categories.
   */
  public async deleteType(typeName: string): Promise<boolean> {
    const target = this.typesState.find((t) => t.name.toLowerCase() === typeName.toLowerCase());
    if (!target) return false;

    try {
      const res = await categoriesApi.deleteType(target.id);
      this.setHierarchy(res.data);
      if (this.selectedNodeState?.type.toLowerCase() === typeName.toLowerCase()) {
        this.selectedNodeState = null;
      }
      console.info(`[cosave:categories] Deleted type: ${typeName} (${target.id})`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to delete type";
      console.error("[cosave:categories] Delete type failed:", err);
      return false;
    }
  }

  /**
   * Creates a new mid-level category under a transaction type.
   */
  public async addCategory(type: TransactionType, name: string): Promise<CategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    try {
      const res = await categoriesApi.createCategory({ type_name: type, name: trimmed });
      this.categoriesState = [...this.categoriesState, res.data];
      this.notify();
      console.info(`[cosave:categories] Added category: ${res.data.name} under ${type}`);
      return res.data;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to add category";
      console.error("[cosave:categories] Add category failed:", err);
    }

    return null;
  }

  /**
   * Creates a new leaf subcategory under a category.
   */
  public async addSubcategory(categoryId: string, name: string): Promise<SubcategoryItem | null> {
    const trimmed = name.trim();
    if (!trimmed) return null;

    try {
      const res = await categoriesApi.createSubcategory({ category_id: categoryId, name: trimmed });
      this.categoriesState = this.categoriesState.map((cat) => {
        if (cat.id === categoryId) {
          return {
            ...cat,
            subcategories: [...cat.subcategories, res.data],
          };
        }
        return cat;
      });
      this.notify();
      console.info(
        `[cosave:categories] Added subcategory: ${res.data.name} to category ${categoryId}`,
      );
      return res.data;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to add subcategory";
      console.error("[cosave:categories] Add subcategory failed:", err);
    }

    return null;
  }

  /**
   * Renames a category.
   */
  public async renameCategory(categoryId: string, newName: string): Promise<boolean> {
    const trimmed = newName.trim();
    if (!trimmed) return false;

    try {
      const res = await categoriesApi.updateCategory(categoryId, trimmed);
      this.setHierarchy(res.data);
      if (this.selectedNodeState?.id === categoryId) {
        this.selectedNodeState.name = trimmed;
      }
      console.info(`[cosave:categories] Renamed category ${categoryId} -> ${trimmed}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to rename category";
      console.error("[cosave:categories] Rename category failed:", err);
      return false;
    }
  }

  /**
   * Renames a leaf subcategory.
   * Can be called as renameSubcategory(subcategoryId, newName)
   * or as renameSubcategory(categoryId, subcategoryId, newName).
   */
  public async renameSubcategory(
    subcategoryIdOrCategoryId: string,
    newNameOrSubcategoryId: string,
    optionalNewName?: string,
  ): Promise<boolean> {
    const subcategoryId = optionalNewName ? newNameOrSubcategoryId : subcategoryIdOrCategoryId;
    const newName = (optionalNewName ?? newNameOrSubcategoryId).trim();
    if (!newName) return false;

    try {
      const res = await categoriesApi.updateSubcategory(subcategoryId, newName);
      this.setHierarchy(res.data);
      if (this.selectedNodeState?.id === subcategoryId) {
        this.selectedNodeState.name = newName;
      }
      console.info(`[cosave:categories] Renamed subcategory ${subcategoryId} -> ${newName}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to rename subcategory";
      console.error("[cosave:categories] Rename subcategory failed:", err);
      return false;
    }
  }

  /**
   * Deletes a category and cascades to its subcategories.
   */
  public async deleteCategory(categoryId: string): Promise<boolean> {
    try {
      const res = await categoriesApi.deleteCategory(categoryId);
      this.setHierarchy(res.data);
      if (
        this.selectedNodeState?.id === categoryId ||
        this.selectedNodeState?.categoryId === categoryId
      ) {
        this.selectedNodeState = null;
      }
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
    subcategoryIdOrCategoryId: string,
    optionalSubcategoryId?: string,
  ): Promise<boolean> {
    const subcategoryId = optionalSubcategoryId ?? subcategoryIdOrCategoryId;

    try {
      const res = await categoriesApi.deleteSubcategory(subcategoryId);
      this.setHierarchy(res.data);
      if (this.selectedNodeState?.id === subcategoryId) {
        this.selectedNodeState = null;
      }
      console.info(`[cosave:categories] Deleted subcategory: ${subcategoryId}`);
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
      const res = await categoriesApi.resetDefaults();
      this.typesState = res.data.types;
      this.categoriesState = res.data.categories;
      this.selectedNodeState = null;
      this.notify();
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
    return projectSankeyGraph(this.typesState, this.categoriesState, activeFilter);
  }
}

export const categoryStore = new CategoryStore();
