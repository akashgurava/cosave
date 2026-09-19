# CoSave Domain Context

Modern, privacy-focused, family-centric financial management platform backed by an embedded Rust/SQLite backend and a responsive SvelteKit interface.

## Execution Environments

- **Development (`./dev.sh dev`)**: Axum backend runs on `:5171`. SvelteKit Vite dev server runs on `:5172` with live HMR and proxies `/api/*` calls to `:5171`.
- **Production (`./dev.sh serve` or Docker)**: Axum serves both `/api/v1/*` REST endpoints and compiled static SPA assets (`frontend/dist`) on port `:5172`.
- **API Inspection & Curls (`./dev.sh curl <endpoint>`)**: Query backend/frontend endpoints (e.g. `./dev.sh curl health`, `./dev.sh curl /api/v1/categories`). Automatically resolves active ports (`:5171` dev backend, `:5172` prod/SPA).
- **Occupied Ports**: If dev or production ports (`:5171`, `:5172`) are occupied during server startup, the maintainer is running the server outside. Do not terminate occupying processes; proceed directly with `./dev.sh curl` or browser testing against the active instance.

## Language

When naming entities, database tables, DTOs, or components, adhere strictly to these terms:

**Family**:
The primary administrative and financial household unit.
_Avoid_: Group, household, team, organization.

**Member**:
An individual belonging to a Family (e.g., parent, child, dependent).
_Avoid_: Profile, persona, occupant.

**User**:
An authenticated account credentials identity that maps to a Member.

**Account**:
A financial account (checking, savings, credit card, loan, investment) owned by a Member or shared across the Family.
_Avoid_: Bank, wallet, ledger.

**Institution**:
The financial institution (bank, credit union, broker) where an Account is held.

**Category Hierarchy**:
The three-tiered classification of financial flows:
- **Type**: Top-level flow classification (`Income`, `Expense`, `Transfer`).
- **Category**: High-level grouping (e.g., `Housing`, `Food`, `Transportation`).
- **Subcategory**: Granular classification bucket (e.g., `Groceries`, `Dining Out`, `Mortgage`).

**Transaction**:
A single financial record of funds moving into or out of an Account.

**Commitment**:
A recurring mandatory expense (subscription, rent, utility, insurance).

**Goal**:
A target savings milestone with a target date and allocated funds.

**Tracer-Bullet Slice**:
An atomic, testable feature increment cutting vertically through all layers, sized for a single context window.
