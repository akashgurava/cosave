/**
 * Authentication, user onboarding, and session security for the application.
 *
 * Frontend presentation and state mirror for the backend auth subsystem:
 *
 * - **Session State**: Reactive `AuthStore` tracking current authenticated identity,
 *   loading states, and administrative roles.
 * - **Dual-Channel Session Handling**: Seamless cookie-based browser session persistence
 *   alongside programmatic token support.
 * - **Contract Envelopes**: Typed `authApi` client with strict runtime schema decoders
 *   (`parseUserDto`, `parseRole`) ensuring zero unvalidated wire payloads.
 */

export * from "./types";
export * from "./api";
export * from "./store.svelte";
