/**
 * Root SPA Route Configuration.
 *
 * Configures SvelteKit to operate strictly in client-side Single Page Application (SPA) mode
 * backed by `@sveltejs/adapter-static` (fallback: `index.html`).
 *
 * - `ssr = false`: Disables server-side rendering; the Rust Axum backend serves the SPA shell
 *   and authoritative REST API endpoints under `/api/v1`.
 * - `prerender = false`: Prevents static route pre-generation at build time, delegating all
 *   routing, session checks, and state hydration dynamically to client-side Rune stores.
 * - `trailingSlash = "never"`: Enforces uniform canonical paths across client-side navigation.
 */

export const prerender = false;
export const ssr = false;
export const trailingSlash = "never";
