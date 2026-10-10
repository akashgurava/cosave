# Transactions Frontend-Backend API Wire-Up & Mock Elimination Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the transactions frontend to the live Rust Axum API server as the authoritative Single Source of Truth (SSOT), align wire DTOs and schema decoders with backend contracts, and completely remove all mock datasets, mock files, and mock imports.

**Architecture:** Maintain strict three-tier separation (`TransactionWireDto` != `Transaction` presentation entity != `TransactionsStore` reactive state). Network responses are received as `unknown` and parsed through zero-dependency runtime decoders (`parseTransactionWireDto`, `parsePaginatedTransactionsWireDto`). The frontend store (`TransactionsStore`) initializes in `idle` state, executes requests through `transactionsApi`, enriches wire DTOs with relational lookups from `familyStore` and `categoryStore`, and exposes fail-fast action methods for mutations.

**Tech Stack:** Svelte 5 (Runes: `$state`, `$derived`, `$props`), TypeScript, Vitest, Rust Axum backend (`/api/v1/transactions`), Tailwind CSS v4.

**Spec:** Backend SSOT implemented in [`backend/src/features/transactions/`](file:///Users/akash/Documents/projects/cosave/backend/src/features/transactions/routes.rs), validated by black-box integration tests in [`backend/tests/api/transaction_routes.rs`](file:///Users/akash/Documents/projects/cosave/backend/tests/api/transaction_routes.rs).

---

## Global Constraints

- **Backend Invariant — Zero Unknown Fields**: Backend request DTOs (`CreateTransactionRequest`, `UpdateTransactionRequest`, `DeleteTransactionRequest`) enforce `#[serde(deny_unknown_fields)]`. Inbound payloads must only contain fields explicitly accepted by the backend. Never send frontend presentation fields (`type`, `typeColor`, `memberId`) in mutation requests.
- **Backend Invariant — Transaction ID Representation**: Transaction IDs are strings (`TEXT` primary keys in SQLite, `String` in Rust `TransactionDto` and `Path<String>`). Update `TransactionId` brand in `$lib/types` to `Brand<string, "TransactionId">`.
- **Backend Invariant — Delete Payload**: Axum route `DELETE /api/v1/transactions/:id` requires a JSON body `{"source": "manual" | "import"}`. API client delete must pass this payload.
- **Frontend Invariant — Zero `any` & Zero Blind `as T`**: All data crossing boundaries must pass through pure runtime decoders throwing `ContractViolationError`.
- **Frontend Invariant — Pure Class-Based Rune Stores**: Store reactive state encapsulated via private `#state = $state<AsyncState<...>>`. Initial state is `idle`, never populated with mock data.
- **Clean Seams**: Completely remove `mock.ts`, `mock_transactions.json`, and `mock.test.ts`. Utility functions (`resolveDatePresetToRange`, `applyFilters`) relocate to `filters.ts`.

---

## Review Focus

1. **Delete Transaction Request Body**: `DELETE /api/v1/transactions/:id` fails with 400 Bad Request if `{"source": "manual" | "import"}` is missing from the HTTP request body. Verify `api.delete` passes `body` correctly through `FetchTransportAdapter` and `MemoryTransportAdapter`.
2. **Serde Unknown Field Rejection on Create & Update**: Sending `type`, `typeColor`, or `memberId` in `POST` or `PATCH` payloads will trigger 422/400 from Axum. Verify request builders strip presentation-only fields.
3. **Paginated Envelope Decoding**: `GET /api/v1/transactions` returns `ApiResponse<PaginatedTransactionsDto>` (`items`, `totalCount`, `page`, `pageSize`, `totalPages`), not a flat array. Verify decoder validates envelope structure and extracts `items`.
4. **Relational Enrichment on Ingestion**: Backend transactions contain `accountId` and `typeId`, but presentation requires `memberId` (from `account.ownerMemberId`), `type` name, and `typeColor` (from `categoryStore.types`). Verify enrichment occurs gracefully even if store metadata is loading.
5. **Session Expiry & 401 Propagation**: Unauthenticated mutations must trigger session invalidation and show authentication modals without crashing the view.

---

## Task Decomposition

### Task 1: API Client Support for DELETE Request Payloads

Allow `api.delete` in `$lib/api/client.ts` to transmit an optional JSON body when endpoints require it (e.g. `DELETE /api/v1/transactions/:id` with `{ source: "manual" }`).

**Files:**
- Modify: [`frontend/src/lib/api/client.ts:20-35, 200-230`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/api/client.ts)
- Modify: [`frontend/src/lib/api/client.test.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/api/client.test.ts)

**Interfaces:**
- Consumes: `RequestOptions<T>`
- Produces: `api.delete<T>(path: string, options?: RequestOptions<T>): Promise<T>` where `options.body` is serialized into `TransportRequest.body`.

- [ ] **Step 1: Write the failing unit test for `api.delete` with body**

Add test to `frontend/src/lib/api/client.test.ts` verifying that `api.delete(url, { body: { source: "manual" } })` passes serialized JSON body to the active transport.

- [ ] **Step 2: Run test to verify it fails**

Run: `./dev.sh ui test client.test.ts`
Expected: FAIL if body is not serialized or passed.

- [ ] **Step 3: Update `RequestOptions` and `executeRequest` in `client.ts`**

Add `body?: unknown;` to `RequestOptions<T>`. In `executeRequest`, resolve `const requestBody = body !== undefined ? body : options.body;` and pass to `executeRequestEnvelope`. In `api.delete`, pass `options.body`.

- [ ] **Step 4: Run test to verify it passes**

Run: `./dev.sh ui test client.test.ts`
Expected: PASS.

---

### Task 2: Align `TransactionId` Brand and Core Types with Backend SSOT

Align `TransactionId` in `frontend/src/lib/types.ts` from `number` to `string` to mirror SQLite `TEXT` and Rust `String` IDs.

**Files:**
- Modify: [`frontend/src/lib/types.ts:25-55`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/types.ts)
- Modify: [`frontend/src/lib/types.test.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/types.test.ts)

**Interfaces:**
- Produces: `export type TransactionId = Brand<string, "TransactionId">;`
- Produces: `export function toTransactionId(raw: unknown): TransactionId;` accepting non-empty strings (and numeric IDs converted to string for backwards safety).

- [ ] **Step 1: Update `types.test.ts` to expect string transaction IDs**

Modify `toTransactionId` tests in `frontend/src/lib/types.test.ts` to assert that `"tx_101"` and `"b6c934f0-1234-4567-89ab-cdef01234567"` pass, and empty strings or non-string/non-integer types throw `ContractViolationError`.

- [ ] **Step 2: Run test to verify it fails**

Run: `./dev.sh ui test types.test.ts`
Expected: FAIL with "TransactionId must be a positive integer".

- [ ] **Step 3: Implement string `TransactionId` in `frontend/src/lib/types.ts`**

Change `export type TransactionId = Brand<string, "TransactionId">;`. Update `toTransactionId(raw: unknown)` to validate non-empty string or positive integer converted to string.

- [ ] **Step 4: Run test to verify it passes**

Run: `./dev.sh ui test types.test.ts`
Expected: PASS.

---

### Task 3: Define Wire DTOs, Decoders, and Mutation Contracts in `features/transactions/types.ts`

Define `TransactionWireDto`, `PaginatedTransactionsWireDto`, pure decoders (`parseTransactionWireDto`, `parsePaginatedTransactionsWireDto`), and strict mutation contracts matching the backend Axum API.

**Files:**
- Modify: [`frontend/src/lib/features/transactions/types.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/types.ts)
- Modify: [`frontend/src/lib/features/transactions/types.test.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/types.test.ts)

**Interfaces:**
- Produces:
  - `TransactionWireDto`: raw wire format matching Rust `TransactionDto` (`id`, `source`, `date`, `description`, `payee`, `amount`, `typeId`, `accountId`, `categoryId`, `subcategoryId`, `notes`, `status`).
  - `PaginatedTransactionsWireDto`: `{ items: readonly TransactionWireDto[]; totalCount: number; page: number; pageSize: number; totalPages: number }`.
  - `parseTransactionWireDto(raw: unknown): TransactionWireDto`.
  - `parsePaginatedTransactionsWireDto(raw: unknown): PaginatedTransactionsWireDto`.
  - `CreateTransactionInput`: Wire payload for `POST /api/v1/transactions` (zero unknown fields).
  - `UpdateTransactionInput`: Wire payload for `PATCH /api/v1/transactions/:id` (requires `source`).
  - `TransactionsTransport` interface returning `PaginatedTransactionsWireDto` and `TransactionWireDto`.

- [ ] **Step 1: Write unit tests in `types.test.ts` for wire decoders**

Add unit tests covering:
- Valid `TransactionWireDto` decoding (null payee, negative amount, optional subcategory and notes).
- Valid `PaginatedTransactionsWireDto` decoding with array of items and pagination metadata.
- Rejection of invalid types, non-integer amounts, malformed date, and non-array `items`.

- [ ] **Step 2: Run test to verify it fails**

Run: `./dev.sh ui test src/lib/features/transactions/types.test.ts`
Expected: FAIL (decoders not yet defined).

- [ ] **Step 3: Implement wire DTOs and decoders in `types.ts`**

Implement `TransactionWireDto`, `PaginatedTransactionsWireDto`, `parseTransactionWireDto`, and `parsePaginatedTransactionsWireDto`. Update `Transaction` entity interface to include `source: "manual" | "import"`. Update `TransactionsTransport` contract.

- [ ] **Step 4: Run test to verify it passes**

Run: `./dev.sh ui test src/lib/features/transactions/types.test.ts`
Expected: PASS.

---

### Task 4: Relocate Pure Filter and Date Utilities to `filters.ts` and Delete Mock Data

Extract non-mock filter helpers (`resolveDatePresetToRange`, `getDatePresetCutoff`, `applyFilters`, `parseCurrencyInput`) into `filters.ts` with unit tests, and delete `mock.ts`, `mock_transactions.json`, and `mock.test.ts`.

**Files:**
- Create: `frontend/src/lib/features/transactions/filters.ts`
- Create: `frontend/src/lib/features/transactions/filters.test.ts`
- Delete: `frontend/src/lib/features/transactions/mock.ts`
- Delete: `frontend/src/lib/features/transactions/mock_transactions.json`
- Delete: `frontend/src/lib/features/transactions/mock.test.ts`
- Modify: [`frontend/src/lib/features/transactions/index.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/index.ts)

**Interfaces:**
- Produces: `resolveDatePresetToRange`, `getDatePresetCutoff`, `applyFilters`, `parseCurrencyInput` in `filters.ts`.

- [ ] **Step 1: Create `filters.ts` and `filters.test.ts`**

Move `resolveDatePresetToRange`, `getDatePresetCutoff`, `applyFilters`, and `parseCurrencyInput` to `filters.ts`. Add thorough tests in `filters.test.ts`.

- [ ] **Step 2: Run tests to verify `filters.test.ts` passes**

Run: `./dev.sh ui test src/lib/features/transactions/filters.test.ts`
Expected: PASS.

- [ ] **Step 3: Remove mock files and clean `index.ts`**

Delete `mock.ts`, `mock_transactions.json`, and `mock.test.ts`. Update `index.ts` to export `* from "./filters"` and remove `export * from "./mock"` and prototype `Variant5`.

---

### Task 5: Implement SSOT API Client and Contract Tests in `api.ts` & `api.test.ts`

Update `transactionsApi` to target the real Axum backend endpoints with exact wire payloads and query parameters, and update contract tests in `api.test.ts`.

**Files:**
- Modify: [`frontend/src/lib/features/transactions/api.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/api.ts)
- Modify: [`frontend/src/lib/features/transactions/api.test.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/api.test.ts)

**Interfaces:**
- Produces: `transactionsApi: TransactionsTransport`:
  - `getTransactions(filters?: TransactionQueryFilters): Promise<PaginatedTransactionsWireDto>`
  - `getTransaction(id: TransactionId | string): Promise<TransactionWireDto>`
  - `createTransaction(payload: CreateTransactionInput): Promise<TransactionWireDto>`
  - `updateTransaction(id: TransactionId | string, payload: UpdateTransactionInput): Promise<TransactionWireDto>`
  - `deleteTransaction(id: TransactionId | string, source?: "manual" | "import"): Promise<null>`

- [ ] **Step 1: Update `api.test.ts` with real backend wire shapes**

Update `api.test.ts` tests:
- `getTransactions` returns paginated envelope `{ items: [...], totalCount: 1, page: 1, pageSize: 20, totalPages: 1 }`.
- `createTransaction` verifies that only backend-accepted fields (`date`, `description`, `payee`, `amount`, `typeId`, `accountId`, `categoryId`, `status`) are sent in the body.
- `updateTransaction` sends `{ source: "manual", ... }` and asserts return of `TransactionWireDto`.
- `deleteTransaction` sends `body: { source: "manual" }` to `DELETE /api/v1/transactions/:id`.

- [ ] **Step 2: Run test to verify it fails**

Run: `./dev.sh ui test src/lib/features/transactions/api.test.ts`
Expected: FAIL.

- [ ] **Step 3: Implement `transactionsApi` in `api.ts`**

Update `api.ts` to serialize query filters (including `q`, `fromDate`, `toDate`, `minAmount`, `maxAmount`, `accountIds`, `typeIds`, `categoryIds`, `statuses`, `page`, `pageSize`) and call endpoints using the new decoders. In `deleteTransaction`, pass `body: { source: source ?? "manual" }`.

- [ ] **Step 4: Run test to verify it passes**

Run: `./dev.sh ui test src/lib/features/transactions/api.test.ts`
Expected: PASS.

---

### Task 6: Refactor `TransactionsStore` to Use Live API, Zero Mock State, and Relational Enrichment

Refactor `TransactionsStore` in `store.svelte.ts`:
- Initial state: `{ status: "idle" }`.
- Enrich incoming `TransactionWireDto` rows with relational lookups (`memberId` from `account.ownerMemberId`, `type` and `typeColor` from `categoryStore.types`).
- Strictly strip presentation fields when dispatching `create` and `update` to satisfy backend `deny_unknown_fields`.
- Handle `delete` passing `source: "manual" | "import"`.
- Export singleton `transactionsStore`.

**Files:**
- Modify: [`frontend/src/lib/features/transactions/store.svelte.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/store.svelte.ts)
- Modify: [`frontend/src/lib/features/transactions/store.test.ts`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/store.test.ts)

**Interfaces:**
- Produces: `export const transactionsStore = new TransactionsStore();`
- Produces: `transactionsStore.load(filters?: TransactionQueryFilters): Promise<void>`
- Produces: `transactionsStore.create(payload: CreateTransactionInput): Promise<Transaction>`
- Produces: `transactionsStore.update(id: TransactionId, updates: Partial<Transaction>): Promise<Transaction>`
- Produces: `transactionsStore.delete(id: TransactionId): Promise<void>`

- [ ] **Step 1: Write store unit tests in `store.test.ts`**

Update `store.test.ts` to test:
- Initial state is `idle` with empty transaction list.
- Calling `load()` transitions to `loading` then `success`, correctly mapping `TransactionWireDto` to `Transaction` using mock relational stores.
- Calling `create()` dispatches clean `CreateTransactionInput` and prepends the new transaction.
- Calling `update()` merges partial updates with the original transaction, includes `source`, strips presentation fields, and updates state.
- Calling `delete()` extracts `source` and dispatches deletion.
- Row drafts management works seamlessly.

- [ ] **Step 2: Run test to verify it fails**

Run: `./dev.sh ui test src/lib/features/transactions/store.test.ts`
Expected: FAIL.

- [ ] **Step 3: Implement `TransactionsStore` in `store.svelte.ts`**

Implement the refactored store, relational mapper `mapWireDtoToTransaction`, mutation cleanup methods, and export `export const transactionsStore = new TransactionsStore();`.

- [ ] **Step 4: Run test to verify it passes**

Run: `./dev.sh ui test src/lib/features/transactions/store.test.ts`
Expected: PASS.

---

### Task 7: Wire Up Route `routes/transactions/+page.svelte` and Components

Update `routes/transactions/+page.svelte` and child components to eliminate mock references and wire directly to `transactionsStore`.

**Files:**
- Modify: [`frontend/src/routes/transactions/+page.svelte`](file:///Users/akash/Documents/projects/cosave/frontend/src/routes/transactions/+page.svelte)
- Modify: [`frontend/src/lib/features/transactions/components/TransactionsView.svelte`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/components/TransactionsView.svelte)
- Modify: [`frontend/src/lib/features/transactions/components/TransactionRow.svelte`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/components/TransactionRow.svelte)
- Modify: [`frontend/src/lib/features/transactions/components/AddTransactionModal.svelte`](file:///Users/akash/Documents/projects/cosave/frontend/src/lib/features/transactions/components/AddTransactionModal.svelte)

- [ ] **Step 1: Update component imports from `../mock` to `../filters` and currency helpers**

In `TransactionRow.svelte` and `AddTransactionModal.svelte`, update imports from `../mock` to `../filters` (for `parseCurrencyInput`). In `TransactionsView.svelte`, update imports from `../mock` to `../filters`.

- [ ] **Step 2: Update `TransactionsView.svelte` to use `transactionsStore` as default**

Default `store = transactionsStore` and bind action handlers (`onAddTransaction`, `onUpdateTransaction`, `onDeleteTransaction`) to `store.create`, `store.update`, and `store.delete` if not explicitly provided as props.

- [ ] **Step 3: Update `routes/transactions/+page.svelte`**

Remove local `let mockTransactions = $state([...])`. Import `transactionsStore`, `familyStore`, `categoryStore`. On `onMount()`, load metadata and transactions:
```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { transactionsStore, TransactionsView } from "$lib/features/transactions";
  import { familyStore } from "$lib/features/family";
  import { categoryStore } from "$lib/features/categories";

  onMount(async () => {
    await Promise.all([familyStore.load(), categoryStore.load()]);
    await transactionsStore.load();
  });
</script>

<svelte:head>
  <title>Transactions — CoSave</title>
</svelte:head>

<div class="container mx-auto max-w-7xl px-4 py-6">
  <TransactionsView store={transactionsStore} />
</div>
```

- [ ] **Step 4: Run typecheck to verify 0 errors**

Run: `./dev.sh ui check`
Expected: 0 errors and 0 warnings.

---

### Task 8: Author Full-Stack Live Integration Test (`transactions.integration.test.ts`)

Create `frontend/src/lib/features/transactions/transactions.integration.test.ts` executing full-stack roundtrips against Axum to verify real API communication, auth cookies, creation, pagination, query filtering, updating, and deletion.

**Files:**
- Create: `frontend/src/lib/features/transactions/transactions.integration.test.ts`

- [ ] **Step 1: Write `transactions.integration.test.ts`**

Follow the established pattern in `family.integration.test.ts`:
1. Authenticates session via register/login.
2. Seeds test account, category, and type if needed.
3. Calls `transactionsApi.createTransaction()` over HTTP via `FetchTransportAdapter`.
4. Calls `transactionsApi.getTransactions()` and verifies pagination envelope.
5. Calls `transactionsApi.updateTransaction()` and verifies updated row.
6. Calls `transactionsApi.deleteTransaction()` and verifies 404 on subsequent get.

- [ ] **Step 2: Run feature tests and verification**

Run: `./dev.sh ui test`
Expected: All Vitest unit tests pass.

- [ ] **Step 3: Run full audit check**

Run: `./dev.sh all check`
Expected: Backend and frontend typechecks and linting pass with 0 errors.

---

## Verification Plan

### Automated Tests
1. **Frontend Unit & Contract Tests**:
   ```bash
   ./dev.sh ui test
   ```
   Verifies all 30 test files pass including `types.test.ts`, `filters.test.ts`, `api.test.ts`, and `store.test.ts`.

2. **Frontend Type Check & Tailwind Lint**:
   ```bash
   ./dev.sh ui check
   ```
   Verifies Svelte 5 runes and TypeScript strict mode with 0 errors.

3. **Backend Unit & Route Tests**:
   ```bash
   ./dev.sh backend test
   ```
   Verifies all 69 backend unit tests and 29 Tier 3 black-box Axum route tests pass.

### Manual Verification
1. Start dev server: `./dev.sh all dev`
2. Navigate to `http://localhost:5172/transactions`.
3. Verify transactions table loads from backend DB without errors.
4. Click "+ Add Transaction", submit a manual transaction, and confirm it appears in the table.
5. Edit an inline field (payee or amount), save draft, and confirm the change persists on refresh.
6. Delete the transaction, confirm deletion.
