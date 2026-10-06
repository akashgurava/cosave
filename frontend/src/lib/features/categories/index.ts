/**
 * Transaction category taxonomy and cashflow classification for the application.
 *
 * Frontend presentation and state mirror for the backend category subsystem:
 *
 * - **3-Tier Hierarchy**: Reactive presentation mirror of top-level transaction types
 *   (Income, Expense, Transfer), categories (Housing, Food & Dining), and subcategories (Rent, Groceries).
 * - **Palette Colors**: Synchronized palette color mappings for financial charts and breakdown badges.
 * - **Visual Projections**: Pure Sankey graph projections (`projectSankeyGraph`) translating hierarchical
 *   entities into ECharts nodes and links.
 * - **Taxonomy State Machine**: Reactive `CategoryStore` maintaining $O(1)$ relational lookup indices,
 *   node selection state, and atomic mutations (create, rename, recolor, delete, reset to defaults).
 * - **Contract Envelopes**: Typed `categoriesApi` service backed by runtime schema decoders.
 */

export * from "./types";
export * from "./api";
export * from "./store.svelte";
export * from "./sankey";
export { default as CategorySankey } from "./components/CategorySankey.svelte";
export { default as CategoryFilterBar } from "./components/CategoryFilterBar.svelte";
export { default as CategorySankeyCard } from "./components/CategorySankeyCard.svelte";
export { default as AddTypeModal } from "./components/AddTypeModal.svelte";
export { default as ResetDefaultsModal } from "./components/ResetDefaultsModal.svelte";
export { default as NodeInspectorModal } from "./components/NodeInspectorModal.svelte";
