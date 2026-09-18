# CONTEXT.md — CoSave Architecture & Domain Context

## 1. Project Overview

**CoSave** is a modern, privacy-focused, family-centric financial management platform.

### Core Philosophy
1. **Family-Centric Finance**: Financial visibility across family members, shared commitments, and savings goals.
2. **Local-First & High-Performance**: Backed by an ultra-fast Rust backend with minimal memory footprint and embedded SQLite persistence.
3. **Apple-Inspired Polish**: A spacious, fluid, OLED-dark desktop and mobile interface built with Svelte 5 Runes, Tailwind CSS v4, and curated micro-interactions.
4. **Self-Contained Container Delivery**: Single multi-stage Alpine container (~21.6 MB) bundling the Axum web server and compiled SvelteKit SPA.

---

## 2. System Architecture

```
                                 [ End User ]
                                      │
                          HTTP Requests (Port 5172)
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
 │  - /family/*         │                           │                      │
 └──────────────────────┘                           └──────────────────────┘
```

> **Execution Environments**:
> - **Development Mode (`./dev.sh dev`)**: Axum backend runs on `:5171`. SvelteKit Vite dev server runs on `:5172` with live HMR and proxies `/api` calls directly to `:5171`.
> - **Production Mode (`./dev.sh serve` or Docker)**: Axum serves both the `/api/v1/*` REST endpoints and static SPA assets (`./frontend/dist`) on a single port (`:5172`).

---

## 3. Feature-First Architecture

Both backend and frontend are organized into symmetrical, self-contained domain modules:

### Backend Structure (`backend/src/`)
```
backend/src/
├── core/                       # Cross-cutting infrastructure
│   ├── db.rs                   # Connection pool setup & schema bootstrap
│   ├── error.rs                # AppError handling
│   ├── response.rs             # ApiResponse<T>, Code, Status enums
│   └── state.rs                # AppState (DbPool, config)
├── features/                   # Self-contained domain modules
│   ├── auth/                   # Users, passwords, and sessions
│   │   ├── mod.rs              # Router export
│   │   ├── db.rs               # SQL queries
│   │   ├── models.rs           # Request/response structs
│   │   └── routes.rs           # Axum HTTP handlers (Zero SQL)
│   └── family/                 # Family members & accounts
│       ├── mod.rs              # Router export
│       ├── db.rs               # SQL queries & SQLite persistence
│       ├── models.rs           # Serde structs matching frontend types.ts
│       └── routes.rs           # Thin HTTP handlers
└── main.rs                     # Server bootstrap & router assembly
```

* **Zero-SQL-in-Routes**: Route handlers only perform HTTP extraction, status codes, and return `ApiResponse<T>`. All SQLx queries live exclusively in `db.rs`.

### Frontend Structure (`frontend/src/`)
```
frontend/src/lib/
├── components/
│   └── ui/                     # Pure upstream shadcn-svelte primitives (IMMUTABLE)
├── features/                   # Self-contained domain modules
│   └── family/                 # Family & accounts feature
│       ├── components/         # Feature components (MemberCard, AccountRow)
│       ├── api.ts              # Typed apiFetch calls
│       ├── types.ts            # TypeScript interfaces matching backend models.rs
│       └── mock.ts             # Prototype mock data for Phase 1
└── ...                         # Shared stores (theme.ts, health.ts)
```

* **Zero Hand-Crafted Primitives**: `lib/components/ui/` is immutable vendor code, installed exclusively via `./dev.sh ui shadcn <component>`.
* **Zero Business Logic in UI**: UI components focus 100% on rendering, layout, and props. Calculations and validation reside exclusively on the backend.

---

## 4. Feature Development Paradigm: Two-Phase Lifecycle

To guarantee best-in-class UX and prevent tangled, bloated pull requests, features progress through two strict phases:

1. **Phase 1: UI Prototyping & UX Freeze (Frontend Only)**:
   - Lead Developer pitches 4 distinct UX design archetypes in the ticket comment.
   - Maintainer reviews and approves the direction.
   - IC builds an interactive sticky switcher at the top of the route (e.g. `/configuration/family`), rendering the 4 variants against `mock.ts`.
   - Zero backend code is written.
   - Maintainer tests on `:5172` and selects the winning design.
2. **Phase 2: Backend SSOT & Wire-up (Full Stack)**:
   - The approved `types.ts` from the winning prototype becomes the authoritative backend contract.
   - IC creates `backend/src/features/<feature>/` (`models.rs`, `db.rs`, `routes.rs`).
   - `frontend/src/lib/features/<feature>/api.ts` swaps mock data for `apiFetch<T>()`.
   - The prototype switcher is removed, leaving the winning design wired to the backend.

---

## 5. Domain Glossary & Language

When naming entities, database tables, DTOs, or Svelte components, use these terms consistently:

- **Family**: The primary administrative and financial household unit.
  - _Avoid_: Group, household, team, organization.
- **Member**: An individual belonging to a Family (e.g., parent, child, dependent).
  - _Avoid_: Profile, persona, occupant.
- **User**: An authenticated account credentials identity that maps to a Member.
- **Account**: A financial account (checking, savings, credit card, loan, investment) owned by a Member or shared across the Family.
  - _Avoid_: Bank, wallet, ledger.
- **Institution**: The financial institution (bank, credit union, broker) where an Account is held.
- **Category Hierarchy**: The three-tiered classification of flows:
  1. **Type**: `Income`, `Expense`, `Transfer`.
  2. **Category**: High-level group (e.g., `Housing`, `Food`, `Transportation`).
  3. **Subcategory**: Specific granular bucket (e.g., `Groceries`, `Dining Out`, `Mortgage`).
- **Transaction**: A single financial record of funds moving into or out of an Account.
- **Commitment**: A recurring mandatory expense (subscription, rent, utility, insurance).
- **Goal**: A target savings milestone with a target date and allocated funds.
- **Tracer-Bullet Slice**: An atomic, testable feature increment cutting vertically through all layers, sized for a single context window.
