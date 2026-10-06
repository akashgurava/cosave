/**
 * Authentication, user onboarding, and session security for the application.
 *
 * Frontend presentation and state mirror for the backend auth subsystem:
 *
 * - **Session State**: Reactive `AuthStore` tracking current authenticated identity,
 *   loading states, and administrative roles.
 * - **Dual-Channel Session Handling**: Seamless cookie-based browser session persistence
 *   alongside programmatic token support.
 * - **Modal & User Controls**: Provides the `AuthModal` dialog for login/signup flows and
 *   `UserMenu` dropdown for profile navigation and revocation.
 * - **Contract Envelopes**: Typed `authApi` client with strict runtime schema decoders
 *   (`parseUserDto`, `parseRole`) ensuring zero unvalidated wire payloads.
 */

export * from "./types";
export * from "./api";
export * from "./store.svelte";
export { default as AuthModal } from "./components/AuthModal.svelte";
export { default as UserMenu } from "./components/UserMenu.svelte";
