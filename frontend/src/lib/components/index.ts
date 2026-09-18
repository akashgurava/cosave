export { default as TopNav } from "./TopNav.svelte";
export { default as AppSidebar } from "./AppSidebar.svelte";
export { default as Sidebar } from "./AppSidebar.svelte";
export { default as StatusBadge } from "./StatusBadge.svelte";
export { default as BackendStatusDot } from "./BackendStatusDot.svelte";
export { default as ThemeSelector } from "./ThemeSelector.svelte";
export { default as FinanceShowcase } from "./FinanceShowcase.svelte";
export { default as MarketingHero } from "./MarketingHero.svelte";

// Feature exports for compatibility
export { AuthModal, UserMenu } from "$lib/features/auth";
export {
  CategorySankey,
  CategoryFilterBar,
  CategorySankeyCard,
  AddTypeModal,
  ResetDefaultsModal,
  NodeInspectorModal,
} from "$lib/features/categories";
