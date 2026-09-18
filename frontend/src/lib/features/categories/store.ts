import { categoriesApi } from "./api";
import {
  type CategoryItem,
  type SankeyLinkData,
  type SankeyNodeData,
  type SelectedCategoryNode,
  type SubcategoryItem,
  type TransactionType,
  type TransactionTypeItem,
} from "./types";

function hexToRgba(hex: string, alpha: number): string {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

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
    const found = this.typesState.find((t) => t.name.toLowerCase() === typeName.toLowerCase());
    const solid = found ? found.color : "#71717a";
    return {
      solid,
      subtle: hexToRgba(solid, 0.25),
      border: solid,
    };
  }

  public isColorUsed(hex: string, excludeTypeName?: string): boolean {
    return this.typesState.some(
      (t) =>
        t.color.toLowerCase() === hex.toLowerCase() &&
        t.name.toLowerCase() !== excludeTypeName?.toLowerCase(),
    );
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

  /**
   * Updates the display color of an existing transaction type.
   */
  public async updateTypeColor(typeName: string, newColor: string): Promise<boolean> {
    const found = this.typesState.find((t) => t.name.toLowerCase() === typeName.toLowerCase());
    if (!found) return false;

    try {
      await categoriesApi.updateTypeColor(found.id, newColor);
      found.color = newColor;
      this.notify();
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
    const index = this.typesState.findIndex((t) => t.name.toLowerCase() === typeName.toLowerCase());
    if (index === -1) return false;
    const target = this.typesState[index];

    try {
      await categoriesApi.deleteType(target.id);
      this.typesState.splice(index, 1);
      this.categoriesState = this.categoriesState.filter(
        (c) => c.type.toLowerCase() !== typeName.toLowerCase(),
      );

      if (this.selectedNodeState?.type.toLowerCase() === typeName.toLowerCase()) {
        this.selectedNodeState = null;
      }
      this.notify();
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
      this.categoriesState.push(res.data);
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
      const category = this.categoriesState.find((c) => c.id === categoryId);
      if (category) {
        category.subcategories.push(res.data);
        this.notify();
      }
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
      await categoriesApi.updateCategory(categoryId, trimmed);
      const category = this.categoriesState.find((c) => c.id === categoryId);
      if (category) {
        category.name = trimmed;
        if (this.selectedNodeState?.id === categoryId) {
          this.selectedNodeState.name = trimmed;
        }
        this.notify();
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
   */
  public async renameSubcategory(
    categoryId: string,
    subcategoryId: string,
    newName: string,
  ): Promise<boolean> {
    const trimmed = newName.trim();
    if (!trimmed) return false;

    try {
      await categoriesApi.updateSubcategory(subcategoryId, trimmed);
      const category = this.categoriesState.find((c) => c.id === categoryId);
      const sub = category?.subcategories.find((s) => s.id === subcategoryId);
      if (sub) {
        sub.name = trimmed;
        if (this.selectedNodeState?.id === subcategoryId) {
          this.selectedNodeState.name = trimmed;
        }
        this.notify();
      }
      console.info(`[cosave:categories] Renamed subcategory ${subcategoryId} -> ${trimmed}`);
      return true;
    } catch (err) {
      this.errorState = err instanceof Error ? err.message : "Failed to rename subcategory";
      console.error("[cosave:categories] Rename subcategory failed:", err);
      return false;
    }
  }

  /**
   * Deletes a category and cascades to its subcategories locally.
   */
  public async deleteCategory(categoryId: string): Promise<boolean> {
    const index = this.categoriesState.findIndex((c) => c.id === categoryId);
    if (index === -1) return false;

    try {
      await categoriesApi.deleteCategory(categoryId);
      this.categoriesState.splice(index, 1);
      if (
        this.selectedNodeState?.id === categoryId ||
        this.selectedNodeState?.categoryId === categoryId
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
   */
  public async deleteSubcategory(categoryId: string, subcategoryId: string): Promise<boolean> {
    try {
      await categoriesApi.deleteSubcategory(subcategoryId);
      const category = this.categoriesState.find((c) => c.id === categoryId);
      if (category) {
        const index = category.subcategories.findIndex((s) => s.id === subcategoryId);
        if (index !== -1) {
          category.subcategories.splice(index, 1);
          if (this.selectedNodeState?.id === subcategoryId) {
            this.selectedNodeState = null;
          }
          this.notify();
        }
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
    const nodes: SankeyNodeData[] = [];
    const links: SankeyLinkData[] = [];
    const addedNodeIds = new Set<string>();

    const isTypeIncluded = (typeName: string): boolean => {
      if (Array.isArray(activeFilter)) {
        if (activeFilter.length === 0 || activeFilter.includes("All")) return true;
        return activeFilter.some((f) => f.toLowerCase() === typeName.toLowerCase());
      }
      if (activeFilter === "All") return true;
      return activeFilter.toLowerCase() === typeName.toLowerCase();
    };

    const filteredCategories = this.categoriesState.filter((c) => isTypeIncluded(c.type));
    const relevantTypes = this.typesState.filter((t) => isTypeIncluded(t.name));

    // Level 0: Types (depth: 0)
    for (const t of relevantTypes) {
      const typeNodeId = `type:${t.name}`;
      const colorObj = this.getTypeColor(t.name);
      if (!addedNodeIds.has(typeNodeId)) {
        addedNodeIds.add(typeNodeId);
        const catsForType = filteredCategories.filter(
          (c) => c.type.toLowerCase() === t.name.toLowerCase(),
        );
        const totalWeight = catsForType.reduce(
          (acc, c) => acc + (c.subcategories.length > 0 ? c.subcategories.length : 1),
          0,
        );
        // For a new type with 0 categories, provide a base weight of 1.5 (+50% from 1) so it's easily clickable.
        // For types with categories, use exact totalWeight so node height matches its ribbon flow perfectly.
        const typeNodeValue = catsForType.length === 0 ? 1.5 : totalWeight;
        nodes.push({
          name: typeNodeId,
          displayName: t.name,
          depth: 0,
          level: "type",
          type: t.name,
          value: typeNodeValue,
          itemStyle: {
            color: colorObj.solid,
            shadowBlur: 4,
            shadowColor: colorObj.solid,
          },
        });
      }
    }

    // Level 1 & Level 2: Categories (depth: 2 -> 40% length) & Subcategories (depth: 5 -> 60% length)
    for (const cat of filteredCategories) {
      const typeNodeId = `type:${cat.type}`;
      const catNodeId = `cat:${cat.id}`;
      const colorObj = this.getTypeColor(cat.type);

      if (!addedNodeIds.has(catNodeId)) {
        addedNodeIds.add(catNodeId);
        nodes.push({
          name: catNodeId,
          displayName: cat.name,
          depth: 2,
          level: "category",
          type: cat.type,
          categoryName: cat.name,
          itemStyle: {
            color: colorObj.solid,
            shadowBlur: 4,
            shadowColor: colorObj.solid,
          },
        });
      }

      if (cat.subcategories.length === 0) {
        links.push({
          source: typeNodeId,
          target: catNodeId,
          value: 1,
          lineStyle: {
            color: colorObj.solid,
            opacity: 0.35,
            shadowBlur: 4,
            shadowColor: colorObj.solid,
          },
        });
      } else {
        links.push({
          source: typeNodeId,
          target: catNodeId,
          value: cat.subcategories.length,
          lineStyle: {
            color: colorObj.solid,
            opacity: 0.35,
            shadowBlur: 4,
            shadowColor: colorObj.solid,
          },
        });

        // Level 2: Subcategories (depth: 5)
        for (const sub of cat.subcategories) {
          const subNodeId = `sub:${cat.id}:${sub.id}`;
          if (!addedNodeIds.has(subNodeId)) {
            addedNodeIds.add(subNodeId);
            nodes.push({
              name: subNodeId,
              displayName: sub.name,
              depth: 5,
              level: "subcategory",
              type: cat.type,
              categoryName: cat.name,
              itemStyle: {
                color: colorObj.solid,
                shadowBlur: 4,
                shadowColor: colorObj.solid,
              },
            });
          }

          links.push({
            source: catNodeId,
            target: subNodeId,
            value: 1,
            lineStyle: {
              color: colorObj.solid,
              opacity: 0.35,
              shadowBlur: 4,
              shadowColor: colorObj.solid,
            },
          });
        }
      }
    }

    return { nodes, links };
  }
}

export const categoryStore = new CategoryStore();
