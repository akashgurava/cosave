import type { CategoryItem, SankeyLinkData, SankeyNodeData, TransactionTypeItem } from "./types";

/**
 * Converts a hex color string to an rgba CSS color with the specified alpha transparency.
 */
export function hexToRgba(hex: string, alpha: number): string {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * Resolves solid, subtle, and border styles for a transaction type.
 */
export function getTypeColor(
  types: readonly TransactionTypeItem[],
  typeName: string,
): { solid: string; subtle: string; border: string } {
  const found = types.find((t) => t.name.toLowerCase() === typeName.toLowerCase());
  const solid = found ? found.color : "#71717a";
  return {
    solid,
    subtle: hexToRgba(solid, 0.25),
    border: solid,
  };
}

/**
 * Checks if a specific color is already used by another transaction type.
 */
export function isColorUsed(
  types: readonly TransactionTypeItem[],
  hex: string,
  excludeTypeName?: string,
): boolean {
  return types.some(
    (t) =>
      t.color.toLowerCase() === hex.toLowerCase() &&
      t.name.toLowerCase() !== excludeTypeName?.toLowerCase(),
  );
}

/**
 * Pure projection that computes the ECharts Sankey graph nodes and links from category hierarchy data.
 */
export function projectSankeyGraph(
  types: readonly TransactionTypeItem[],
  categories: readonly CategoryItem[],
  activeFilter: string | string[] = "All",
): {
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

  const filteredCategories = categories.filter((c) => isTypeIncluded(c.type));
  const relevantTypes = types.filter((t) => isTypeIncluded(t.name));

  // Level 0: Types (depth: 0)
  for (const t of relevantTypes) {
    const typeNodeId = `type:${t.name}`;
    const colorObj = getTypeColor(types, t.name);
    if (!addedNodeIds.has(typeNodeId)) {
      addedNodeIds.add(typeNodeId);
      const catsForType = filteredCategories.filter(
        (c) => c.type.toLowerCase() === t.name.toLowerCase(),
      );
      const totalWeight = catsForType.reduce(
        (acc, c) => acc + (c.subcategories.length > 0 ? c.subcategories.length : 1),
        0,
      );
      // For a new type with 0 categories, provide a base weight of 1.5 so it is easily clickable.
      const typeNodeValue = catsForType.length === 0 ? 1.5 : totalWeight;
      nodes.push({
        name: typeNodeId,
        displayName: t.name,
        depth: 0,
        level: "type",
        type: t.name,
        value: typeNodeValue,
        entity: {
          id: t.id,
          type: t.name,
          kind: "type",
          name: t.name,
        },
        itemStyle: {
          color: colorObj.solid,
          shadowBlur: 4,
          shadowColor: colorObj.solid,
        },
      });
    }
  }

  // Level 1 & Level 2: Categories (depth: 2) & Subcategories (depth: 5)
  for (const cat of filteredCategories) {
    const typeNodeId = `type:${cat.type}`;
    const catNodeId = `cat:${cat.id}`;
    const colorObj = getTypeColor(types, cat.type);

    if (!addedNodeIds.has(catNodeId)) {
      addedNodeIds.add(catNodeId);
      nodes.push({
        name: catNodeId,
        displayName: cat.name,
        depth: 2,
        level: "category",
        type: cat.type,
        categoryName: cat.name,
        entity: {
          id: cat.id,
          type: cat.type,
          kind: "category",
          name: cat.name,
          parentName: cat.type,
        },
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
            entity: {
              id: sub.id,
              type: cat.type,
              kind: "subcategory",
              name: sub.name,
              parentName: cat.name,
              categoryId: cat.id,
            },
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
