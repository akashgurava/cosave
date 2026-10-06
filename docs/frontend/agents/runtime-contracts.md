# Frontend Runtime Contracts & Boundary Decoders

Guidelines for parsing external boundaries, validating wire data, and enforcing API contracts in the frontend.

## Trigger & Scope

Consult this guide whenever implementing network calls, handling LocalStorage/URL state, creating `types.ts`, or implementing `api.ts` transport adapters.

## 1. "Parse, Don't Validate" at the Boundary

In Rust, `serde` validates types upon ingestion, ensuring internal domain types are infallible. In TypeScript, we enforce the identical guarantee:

* Network data is typed as `unknown` upon receipt.
* Data is immediately transformed into domain types through dedicated pure decoder functions: `parseX(raw: unknown): T`.
* Blind JSON casting (`as T`) is strictly forbidden.

## 2. Authoring Runtime Decoders

Decoders reside in `frontend/src/lib/features/<feature>/types.ts` alongside domain interfaces:

```ts
import { ContractViolationError, isObject } from "$lib/api";

export interface CategoryItem {
  readonly id: number;
  readonly name: string;
  readonly subcategories: readonly SubcategoryItem[];
}

export function parseCategoryItem(raw: unknown): CategoryItem {
  if (isObject(raw) === false) {
    throw new ContractViolationError("CategoryItem payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("CategoryItem.id must be an integer", raw);
  }
  if (typeof raw.name !== "string" || raw.name.trim().length === 0) {
    throw new ContractViolationError("CategoryItem.name must be a non-empty string", raw);
  }
  if (Array.isArray(raw.subcategories) === false) {
    throw new ContractViolationError("CategoryItem.subcategories must be an array", raw);
  }

  return {
    id: raw.id,
    name: raw.name,
    subcategories: Object.freeze(raw.subcategories.map(parseSubcategoryItem)),
  };
}
```

## 3. Error Handling Contract

Backend API responses wrap error payloads:
```json
{
  "code": 409,
  "status": "TYPE_ALREADY_EXISTS",
  "data": {
    "action": "CONFIG.CATEGORIES.CREATE_TYPE.ALREADY_EXISTS",
    "message": "Transaction type 'Crypto' already exists."
  }
}
```

The frontend API layer converts non-2xx responses into `ApiError` instances:
- `error.action`: The exact compile-time action string from Rust.
- `error.message`: The human-readable error description.
- `error.httpStatus`: The HTTP status code.

Components and stores extract `{ action, message }` to surface accurate, targeted user feedback.
