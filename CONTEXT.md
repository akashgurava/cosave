# CONTEXT.md — CoSave Architecture & Project Context

## 1. Project Overview

**CoSave** is a modern, privacy-focused, family-centric financial management platform. 

### Why CoSave?
Existing open-source personal finance platforms (such as Firefly III) are predominantly structured around single-user accounts and legacy web server templates. In contrast, **CoSave** is designed from the ground up for:
1. **Family-Centric Finance & Savings**: Comprehensive, unified financial visibility across family members, shared commitments, and savings goals.
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
 │  - /auth/*           │                           │   - /assets/*        │
 │  - Standard Envelopes│                           │                      │
 └──────────────────────┘                           └──────────────────────┘
```

> **Dual Execution Environments**:
> - **Development Mode (`./dev.sh dev`)**: Axum backend runs on `:3000` with debug tracing. SvelteKit Vite dev server runs on `:5173` with live HMR and proxies `/api` calls directly to `:3000`.
> - **Production / Container Mode (`./dev.sh serve` or Docker)**: Axum serves both the `/api/v1/*` REST endpoints and the static compiled SPA assets (`./frontend/dist`) on a single port (`:3000`).

### Core Architecture: Backend as Single Source of Truth

The Rust backend is the authoritative **Single Source of Truth (SSOT)** across the entire system:
1. **Data**: The backend owns all schemas, models, database persistence, and validation constraints.
2. **API Routes**: All route definitions, endpoints, request contracts, and response envelopes originate from and are enforced by the backend.
3. **Business Logic**: Core calculations (e.g. spend categorization, savings metrics, expense reduction analysis, and family budget tracking) are implemented exclusively in Rust backend services. Business logic must never be duplicated or executed independently on the frontend.
4. **Frontend as an Interchangeable Mirror**: The SvelteKit frontend functions strictly as a reactive, presentation-layer mirror of backend state and contracts, handling UI rendering, accessibility, navigation, and user interaction.

### Feature Development Paradigm: UI-Driven Demand, API-First Implementation

1. **UI-Driven Demand**: Feature development is initiated from direct user interaction with the web interface. The user identifies missing visual elements, workflows, or analytical capabilities on the screen and requests implementation from the AI assistant.
2. **Backend-First Engineering**: Rather than implementing features client-side, the AI builds the solution inside-out:
   - **Step 1 (Data & Storage)**: Define SQLite tables, foreign keys, and SQLx queries (`backend/src/db.rs`).
   - **Step 2 (Domain Logic)**: Implement calculations, aggregations, and business rules in Rust services.
   - **Step 3 (API Contract)**: Expose clean, strongly-typed endpoints under `/api/v1/*` using standardized `ApiResponse<T>` envelopes.
   - **Step 4 (UI Mirror)**: Render the backend-validated data in Svelte components using `apiFetch<T>()`.
3. **Headless Server & Swappable Frontends**: The API server is completely autonomous and headless. The frontend is treated as a flexible, swappable presentation layer. This guarantees that frontends can be redesigned, refactored, or expanded to mobile apps (e.g. Flutter, React Native, Swift), desktop apps, or CLI tools without changing backend business logic or database schemas.

### Backend (`backend/`)
- **Framework**: `axum` (0.8) on `tokio` runtime with `axum-extra` (cookie jar).
- **Database & Persistence**: Embedded SQLite managed via asynchronous `sqlx::SqlitePool` with WAL journal mode, foreign key enforcement, and automatic schema migration on startup (`backend/src/db.rs`).
- **Authentication & Security**: Argon2id password hashing with cryptographically secure random salts, session tokens stored in SQLite with expiration timestamps, and HTTP-only cookie distribution.
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
- **API Endpoints**:
  - `GET /health` — Service health beacon and version info.
  - `POST /api/v1/auth/register` — Create new user credentials.
  - `POST /api/v1/auth/login` — Authenticate and establish session cookie.
  - `GET /api/v1/auth/me` — Retrieve active session profile.
  - `POST /api/v1/auth/logout` — Clear session token.
- **Logging**:
  - `info` level by default (`cosave=info,tower_http=info`).
  - `-v` / `--verbose` flag toggles `debug` logging.

### Frontend (`frontend/`)
- **Framework**: SvelteKit 2 + Svelte 5 in Runes mode (`$state`, `$derived`, `$effect`, `$props`) using `@sveltejs/adapter-static` for static SPA distribution.
- **Routing**: Client-side routing with `/` (Marketing Hero / Authenticated Family Dashboard) and `/settings` (Settings shell), using typesafe `resolve()` from `$app/paths`.
- **Styling**: Tailwind CSS v4 via `@tailwindcss/vite`, strictly using canonical classes (`border-(--var)`, `size-8`) and CSS variable design tokens.
- **Theme System**: Dual OLED dark mode (`#000000`) and pure light mode (`#ffffff`) with financial emerald green accents and theme-adaptive primary buttons.
- **Reactive Stores**:
  - `authStore` (`frontend/src/lib/auth.ts`) — Session lifecycle, user profile, login, registration, and logout.
  - `themeStore` (`frontend/src/lib/theme.ts`) — Dark/light/system theme resolution with localStorage persistence.
  - `healthStore` (`frontend/src/lib/health.ts`) — Polling backend reachability beacon with timer cleanup.
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
| **Embedded SQLite Database** | Complete | Asynchronous SQLx connection pool, WAL mode, auto-migrations. |
| **Authentication & Sessions** | Complete | Argon2id password hashing, cookie-based session management. |
| **API Envelope & Enums** | Complete | `Code` and `Status` enums with `ApiResponse` builders. |
| **SvelteKit 2 Shell** | Complete | Svelte 5 runes, multi-route support (`/` and `/settings`), live status badge. |
| **Marketing Hero & Dashboard** | Complete | Unauthenticated product marketing hero & authenticated family finance shell. |
| **Design System & Themes** | Complete | OLED black / pure white, emerald financial accents, theme-adaptive buttons. |
| **Tailwind CSS v4** | Complete | Configured with Vite plugin and canonical class linting. |
| **Strict Linting & CI** | Complete | Prettier, ESLint, svelte-check, clippy, canonical checks. |
| **Multi-Stage Container** | Complete | Fully tested image building and serving live requests. |
| **Developer CLI (`./dev.sh`)** | Complete | Fast build (`fbuild`), full verification (`full`), flint (`flint`), doctor (`doctor`), clean (`clean`), shadcn wrapper, dev server, and production serve. |

---

## 4. Immediate Next Steps / Roadmap

1. **Family Accounts & Member Modeling**: Define domain schemas and SQLite tables for family members, bank accounts, and transaction records.
2. **Statement Ingestion**: Ingest banking statements and spreadsheets using Rust parsing (e.g. `calamine` for Excel, `csv`).
3. **Spend Categorization & Rules Engine**: Configurable matching rules in Rust for auto-categorization, tagging, and merchant cleanup.
4. **Budgeting & Expense Reduction Metrics**: Calculate spend patterns, category budgets, and savings metrics in Rust backend services.
5. **Configuration & Management UI**: Build the settings and management views for accounts, family members, and import workflows.
