# CoSave Domain Context

Modern, privacy-focused, family-centric financial management platform backed by an embedded Rust/SQLite backend and a responsive SvelteKit interface.

## Execution Environments

- **Development (`./dev.sh dev`)**: Axum backend runs on `:5171`. SvelteKit Vite dev server runs on `:5172` with live HMR and proxies `/api/*` calls to `:5171`.
- **Production (`./dev.sh serve` or Docker)**: Axum serves both `/api/v1/*` REST endpoints and compiled static SPA assets (`frontend/dist`) on port `:5172`.
- **API Inspection & Curls (`./dev.sh curl <endpoint>`)**: Query backend/frontend endpoints (e.g. `./dev.sh curl health`, `./dev.sh curl /api/v1/config/hierarchy`). Automatically resolves active ports (`:5171` dev backend, `:5172` prod/SPA).
- **Occupied Ports**: If dev or production ports (`:5171`, `:5172`) are occupied during server startup, the maintainer is running the server outside. Do not terminate occupying processes; proceed directly with `./dev.sh curl` or browser testing against the active instance.

## Language

When naming entities, database tables, DTOs, or components, adhere strictly to these terms:

**Family**:
The primary administrative and financial household unit. A family operates in a single base currency (`currency_id`), which is strictly inherited by all owned financial accounts. Cross-currency operations and mixed-currency conversions are disallowed.
_Avoid_: Group, household, team, organization.

**Member**:
An individual belonging to a Family (e.g., parent, child, dependent).
_Avoid_: Profile, persona, occupant.

**User**:
An authenticated account credentials identity that maps to a Member. The login credential handle is always **username**.
_Avoid_: User_name, name (for credentials), profile_name.

**Account**:
A financial account (checking, savings, credit card, loan, investment) owned by a Member or shared across the Family. Always operates in the household's base currency.
- **Bank Account**: Depository account characterized by `bank_name`, `account_name`, `last4`, and `available_balance` (`available_balance: i64` in integer minor units).
- **Credit Card**: Revolving credit facility characterized by `bank_name`, `card_name`, `last4`, `credit_limit` (`credit_limit: i64`), and user-updatable `available_credit` (`available_credit: i64`), with derived `outstanding_balance` (`outstanding_balance = credit_limit - available_credit`).
_Avoid_: Bank, wallet, ledger.

**Money & Minor Units**:
All monetary amounts across the stack are stored, computed, and transferred strictly as 64-bit integer Minor Units (`AmountMinorUnits` / `MinorUnits`), never floating-point. The integer value is scaled according to the currency's ISO 4217 decimal scale (scale 0 for JPY/KRW, scale 2 for USD/EUR/INR, scale 3 for KWD/BHD). Form inputs split on decimal characters without floating-point arithmetic.
_Avoid_: Cent/cents (as a generic synonym for minor units), dollar (as a generic synonym for money).

**Institution**:
The financial institution (bank, credit union, broker) where an Account is held.

**Category Hierarchy**:
The three-tiered classification of financial flows:
- **Type**: Top-level flow classification (`Income`, `Expense`, `Transfer`). Links strictly to a curated palette `color_id` (`i64`).
- **Category**: High-level grouping (e.g., `Housing`, `Food`, `Transportation`) scoped under a parent transaction type.
- **Subcategory**: Granular classification bucket (e.g., `Groceries`, `Dining Out`, `Mortgage`) scoped under a parent category.
- **REST & Route Architecture**:
  - `GET /api/v1/config/hierarchy`: Single source of truth query assembling the 3-tier tree from view `v_category_hierarchy` (columns in canonical order: `type_color_id, type_color, type_id, type_name, type_sort_order, category_id, category_name, category_sort_order, subcategory_id, subcategory_name, subcategory_sort_order`). Top-level transaction types are queried exclusively through this endpoint.
  - `GET /api/v1/config/categories/colors`: Curated palette query returning available color options (`id: i64, name: String, hex: String`).
  - `POST /api/v1/config/hierarchy/reset`: Administrative reset restoring template defaults, returning `ApiResponse<()>` (`data: null`).
  - `/api/v1/config/categories/*`: Granular mutations (`POST`, `PATCH`, `DELETE` for types, categories, and subcategories) adhering strictly to Command-Query Separation (CQS) by returning created/updated domain entities or `ApiResponse<()>` without read amplification. All mutations calculate sequential sort orders inline within single-shot atomic SQL statements.

**Transaction**:
A single, atomic financial record of funds moving into or out of an Account at a specific point in time. Each transaction is atomic; split transactions (parent-child hierarchies or sub-transactions) are intentionally unrepresented in the domain model.
- Associated with an Account, an attributing Member, a transaction Type (`Income`, `Expense`, `Transfer`), and an optional Category and Subcategory.
- Monetary value is stored and computed strictly as integer `MinorUnits` in the Family's base currency.
- Master transaction records in the ledger are clean, read-optimized representations whose identity originates from a `Transaction Source`.
- _Avoid_: Ledger entry, sub-transaction, split, line item.

**Transfer**:
A movement of funds between two family accounts. Modeled as two separate, atomic transactions classified under the `Transfer` transaction type (an outflow leg from the source account and an inflow leg to the destination account).
- Both transfer legs are excluded from household cashflow (income vs. expense) and net-worth change calculations.
- Does not require rigid 1:1 cross-account amount matching or relational link pairing, naturally accommodating transfer fees, wire charges, and differing settlement times.
- _Avoid_: Split transfer, double-entry pairing link, internal movement.

**Transaction Source**:
The authoritative provenance and identity registry that assigns a canonical identifier to every transaction and tracks its lifecycle origin across two parallel ingestion streams:
- **Statement Import**: Transactions ingested in batch from an institution statement through raw row capture (`raw_statement_rows`) into staging (`staging_transactions`). Raw payload is preserved verbatim and unconditionally.
- **Manual Entry**: Transactions recorded directly by a user (`manual_transactions`).
- **Reconciliation & Precedence**: `transaction_sources` maintains a discriminator (`source_type: 'import' | 'manual'`) as the single source of truth for the active stream. When a user reconciles a statement transaction with an existing manual transaction or edits an imported record, `transaction_sources` points to the authoritative stream while preserving provenance links across both origins.
- _Avoid_: Import log, transaction history, audit table.

**Commitment**:
A recurring mandatory expense (subscription, rent, utility, insurance).

**Goal**:
A target savings milestone with a target date and allocated funds.

**Tracer-Bullet Slice**:
An atomic, testable feature increment cutting vertically through all layers, sized for a single context window.
