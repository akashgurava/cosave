/**
 * Family unit, member rosters, and financial accounts feature module.
 *
 * Coordinates multi-member family management and account tracking, mirroring the
 * authoritative Rust backend subsystem (`backend/src/features/family/mod.rs`):
 *
 * - **Family Entity & Base Currency**: Manages family metadata, display name, and
 *   authoritative base currency with dynamic regional inference and fallback.
 * - **Member Rosters & Lifecycle**: Supports adding, renaming, and removing family members
 *   with cascade cleanup across member-owned accounts.
 * - **Account Ownership & Instrument Discrimination**: Discriminated union handling for
 *   depository bank accounts and revolving credit cards (`Account = BankAccount | CreditCardAccount`),
 *   enforcing member ownership, independent currencies, and scale-aware minor units (`MinorUnits`).
 * - **Reactive Presentation Store**: Encapsulates state machine transitions (`FamilyStore`)
 *   with $O(1)$ reactive indices (`#memberByIdMap`, `#accountByIdMap`, `#currencyByIdMap`),
 *   and invariant-asserting getters (`requireMember`, `requireAccount`, `requireCurrency`).
 */

export * from "./types";
export * from "./currency";
export * from "./api";
export * from "./store.svelte";
export { default as FamilyView } from "./components/FamilyView.svelte";
export { default as MemberModal } from "./components/MemberModal.svelte";
export { default as AccountModal } from "./components/AccountModal.svelte";
export { default as CreateFamilyModal } from "./components/CreateFamilyModal.svelte";
