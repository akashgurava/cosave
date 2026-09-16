# CONTEXT.md — CoSave Architecture & Project Context

## 1. Project Overview

**CoSave** is a modern, privacy-focused, family-centric financial management platform. 

### Why CoSave?
Existing open-source personal finance platforms (such as Firefly III) are predominantly structured around single-user accounts and legacy web server templates. In contrast, **CoSave** is designed from the ground up for:
1. **Family Hive Modeling**: Comprehensive, unified financial visibility across family members, joint accounts, and shared commitments.
2. **Local-First & High-Performance**: Backed by a lightweight, lightning-fast Rust backend with minimal memory footprint.
3. **Modern, Reactive UI**: A fluid desktop and mobile-accessible interface powered by Svelte 5 Runes and Tailwind CSS v4.
4. **Self-Contained Container Delivery**: Distributed as a tiny (~21.6 MB) Alpine multi-stage Docker container that bundles both the web server and the static UI assets.

---

## 2. System Architecture

```
                                 [ End User ]
                                      │
                         HTTP Requests (Port 3000)
                                      ▼
                       ┌─────────────────────────────┐
                       │    Rust Axum Web Server     │
                       └──────────────┬──────────────┘
                                      │
            ┌─────────────────────────┴─────────────────────────┐
            ▼                                                   ▼
 ┌──────────────────────┐                           ┌──────────────────────┐
 │  REST API (/api/v1)  │                           │   Static Assets (UI) │
 │  - /health           │                           │   - index.html (SPA) │
 │  - Standard Envelopes│                           │   - /assets/*        │
 └──────────────────────┘                           └──────────────────────┘
```

### Backend (`backend/`)
- **Framework**: `axum` on `tokio` runtime.
- **Middleware**: `tower-http` with `CorsLayer`, `TraceLayer`, and `ServeDir` fallback to `index.html`.
- **Response Protocol**:
  - All JSON endpoints adhere to the envelope:
    ```json
    {
      "code": 0,
      "status": "HEALTHY",
      "data": { ... }
    }
    ```
  - Strongly-typed `Code` and `Status` enums defined in `backend/src/response.rs`.
- **Logging**:
  - `info` level by default (`cosave=info,tower_http=info`).
  - `-v` / `--verbose` flag toggles `debug` logging.

### Frontend (`frontend/`)
- **Framework**: SvelteKit 2 + Svelte 5 in Runes mode (`$state`, `$derived`, `$effect`) using `@sveltejs/adapter-static` for static SPA distribution.
- **Routing**: Client-side routing with `/` (Home with live backend indicator badge) and `/settings` (Settings shell), using typesafe `resolve()` from `$app/paths`.
- **Styling**: Tailwind CSS v4 via `@tailwindcss/vite`, strictly using canonical classes (`border-(--var)`, `size-8`) and CSS variable design tokens.
- **Component Architecture**: Components located in `frontend/src/lib/components/` with barrel exports (`index.ts`) accessible via the `$components` path alias.
- **Client API Layer** (`frontend/src/lib/api.ts`):
  - Typed `Code` and `Status` enums/const objects with constructors (`Code.zero()`, `Status.healthy()`).
  - Strict extractors (`extractApiResponse`, `extractData`, `extractCode`, `extractStatus`) with explicit error throws for unexpected codes or statuses.
  - `apiFetch<T>()` utility for safe, typed HTTP requests.

### Containerization (`Dockerfile`)
- Multi-stage Alpine container:
  1. **Frontend Builder (`node:24-alpine`)**: Uses `pnpm` via Corepack to compile SvelteKit into `frontend/dist`.
  2. **Backend Builder (`rust:alpine`)**: Compiles `cosave` in `--release` mode with cargo dependency layer caching.
  3. **Runtime (`alpine:3.21`)**: Runs as non-root `appuser:appgroup`, exposing port `3000` with a binary + static asset footprint of only **~21.6 MB**.

---

## 3. Current Project State

| Feature / Area | Status | Notes |
| :--- | :--- | :--- |
| **Rust Axum Server** | Complete | Listens on port 3000, serves static assets and API routes. |
| **API Envelope & Enums** | Complete | `Code` and `Status` enums with `ApiResponse` builders. |
| **SvelteKit 2 Shell** | Complete | Svelte 5 runes, multi-route support (`/` and `/settings`), live status badge. |
| **Tailwind CSS v4** | Complete | Configured with Vite plugin and canonical class linting. |
| **Strict Linting & CI** | Complete | Prettier, ESLint, svelte-check, clippy, canonical checks. |
| **Multi-Stage Container** | Complete | Fully tested image building and serving live requests. |
| **Developer CLI (`./dev.sh`)** | Complete | Scoped (`backend`, `ui`) & full-stack (`lint`, `format`, `check`, `test`, `build`, `serve`). |

---

## 4. Immediate Next Steps / Roadmap

1. **Family Hive & Account Modeling**: Define core domain models for families, members, bank accounts, and transaction types.
2. **Statement Ingestion**: Ingest banking statements and spreadsheets using Rust parsing (e.g. `calamine` for Excel, `csv`).
3. **Categorization & Rules Engine**: Configurable matching rules for auto-categorization and tagging.
4. **Database Persistence**: Embed local database storage (e.g., SQLite via SQLx or Diesel) for persistent transactions and configuration.
5. **Configuration UI**: Complete the Settings view for managing hives, rules, and accounts.
