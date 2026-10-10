import { ContractViolationError, isObject } from "$lib/api";
import {
  toMinorUnits,
  toTransactionId,
  toTypeId,
  type AccountId,
  type CategoryId,
  type MemberId,
  type MinorUnits,
  type SubcategoryId,
  type TransactionId,
  type TypeId,
} from "$lib/types";

export type { AccountId, CategoryId, MemberId, MinorUnits, SubcategoryId, TransactionId, TypeId };

export type TransactionType = string;
export type TransactionStatus = "cleared" | "pending";
export type TransactionSource = "manual" | "import";

/**
 * Raw wire DTO representing a single transaction in Axum backend response.
 */
export interface TransactionWireDto {
  readonly id: string;
  readonly source: TransactionSource;
  readonly date: string; // ISO date 'YYYY-MM-DD'
  readonly description: string | null;
  readonly payee: string | null;
  readonly amount: number; // integer minor units
  readonly typeId: number;
  readonly accountId: number;
  readonly categoryId: number;
  readonly subcategoryId: number | null;
  readonly notes: string | null;
  readonly status: string;
}

/**
 * Paginated wire DTO envelope returned by GET /api/v1/transactions.
 */
export interface PaginatedTransactionsWireDto {
  readonly items: readonly TransactionWireDto[];
  readonly totalCount: number;
  readonly page: number;
  readonly pageSize: number;
  readonly totalPages: number;
}

/**
 * Presentation-layer domain entity used by UI components and reactive stores.
 */
export interface Transaction {
  readonly id: TransactionId;
  readonly source: TransactionSource;
  readonly date: string; // ISO date 'YYYY-MM-DD'
  readonly description: string | null;
  readonly payee: string;
  readonly amount: MinorUnits;
  readonly typeId: TypeId;
  readonly type: string; // Dynamic type name from hierarchy (e.g. "Income", "Expense", "Transfer", "Invest")
  readonly typeColor: string; // Palette color hex from hierarchy
  readonly memberId: MemberId;
  readonly accountId: AccountId;
  readonly toAccountId?: AccountId;
  readonly categoryId: CategoryId;
  readonly subcategoryId?: SubcategoryId;
  readonly notes?: string;
  readonly status: TransactionStatus;
}

export type DatePreset = "all" | "1d" | "3d" | "7d" | "1m" | "3m" | "6m" | "1y" | "custom";
export type AmountPreset = "all" | "lt100" | "lt500" | "lt1000" | "lt2000" | "gte2000" | "custom";
export type SortField =
  | "date"
  | "description"
  | "payee"
  | "amount"
  | "type"
  | "member"
  | "account"
  | "category"
  | "status";
export type SortDirection = "asc" | "desc";

export interface TransactionFilters {
  readonly searchQuery: string;
  readonly datePreset: DatePreset;
  readonly startDate?: string;
  readonly endDate?: string;
  readonly customDateFrom?: string;
  readonly customDateTo?: string;
  readonly amountPreset: AmountPreset;
  readonly customAmountMin?: number;
  readonly customAmountMax?: number;
  readonly selectedMemberIds: readonly MemberId[];
  readonly selectedAccountIds: readonly AccountId[];
  readonly selectedTypeIds: readonly TypeId[];
  readonly selectedCategoryIds: readonly CategoryId[];
  readonly selectedSubcategoryIds: readonly SubcategoryId[];
  readonly selectedStatuses: readonly TransactionStatus[];
}

/**
 * Mutation payload for creating a transaction on the backend.
 * Rejects unknown fields per Axum #[serde(deny_unknown_fields)].
 */
export interface CreateTransactionInput {
  readonly date: string;
  readonly description?: string | null;
  readonly payee?: string;
  readonly amount: MinorUnits | number;
  readonly typeId: TypeId | number;
  readonly accountId: AccountId | number;
  readonly categoryId: CategoryId | number;
  readonly subcategoryId?: SubcategoryId | number | null;
  readonly notes?: string | null;
  readonly status?: TransactionStatus;
}

/**
 * Mutation payload for modifying an existing transaction on the backend.
 * Requires source discriminator ("manual" | "import").
 */
export interface UpdateTransactionInput {
  readonly source: TransactionSource;
  readonly date: string;
  readonly description?: string | null;
  readonly payee?: string;
  readonly amount: MinorUnits | number;
  readonly typeId: TypeId | number;
  readonly accountId: AccountId | number;
  readonly categoryId: CategoryId | number;
  readonly subcategoryId?: SubcategoryId | number | null;
  readonly notes?: string | null;
  readonly status?: TransactionStatus;
}

/**
 * Validates and decodes raw JSON into a TransactionWireDto.
 */
export function parseTransactionWireDto(raw: unknown): TransactionWireDto {
  if (!isObject(raw)) {
    throw new ContractViolationError("TransactionWireDto payload must be an object", raw);
  }

  if (typeof raw.id !== "string" || raw.id.trim().length === 0) {
    if (typeof raw.id === "number" && Number.isInteger(raw.id) && raw.id > 0) {
      // Allow legacy numeric IDs converted to string
    } else {
      throw new ContractViolationError("TransactionWireDto.id must be a non-empty string", raw);
    }
  }
  const id = String(raw.id).trim();

  const source: TransactionSource =
    raw.source === "import" ? "import" : raw.source === "manual" ? "manual" : "manual";

  if (typeof raw.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(raw.date)) {
    throw new ContractViolationError(
      "TransactionWireDto.date must be an ISO date string (YYYY-MM-DD)",
      raw,
    );
  }

  const description = typeof raw.description === "string" ? raw.description : null;
  const payee = typeof raw.payee === "string" ? raw.payee : null;

  if (typeof raw.amount !== "number" || !Number.isInteger(raw.amount)) {
    throw new ContractViolationError(
      "TransactionWireDto.amount must be an integer (minor units)",
      raw,
    );
  }

  if (typeof raw.typeId !== "number" || !Number.isInteger(raw.typeId) || raw.typeId <= 0) {
    throw new ContractViolationError("TransactionWireDto.typeId must be a positive integer", raw);
  }

  if (typeof raw.accountId !== "number" || !Number.isInteger(raw.accountId) || raw.accountId <= 0) {
    throw new ContractViolationError(
      "TransactionWireDto.accountId must be a positive integer",
      raw,
    );
  }

  if (
    typeof raw.categoryId !== "number" ||
    !Number.isInteger(raw.categoryId) ||
    raw.categoryId <= 0
  ) {
    throw new ContractViolationError(
      "TransactionWireDto.categoryId must be a positive integer",
      raw,
    );
  }

  const subcategoryId =
    typeof raw.subcategoryId === "number" &&
    Number.isInteger(raw.subcategoryId) &&
    raw.subcategoryId > 0
      ? raw.subcategoryId
      : null;

  const notes =
    typeof raw.notes === "string" && raw.notes.trim().length > 0 ? raw.notes.trim() : null;

  const status = raw.status === "pending" ? "pending" : "cleared";

  return Object.freeze({
    id,
    source,
    date: raw.date,
    description,
    payee,
    amount: raw.amount,
    typeId: raw.typeId,
    accountId: raw.accountId,
    categoryId: raw.categoryId,
    subcategoryId,
    notes,
    status,
  });
}

/**
 * Validates and decodes raw JSON into PaginatedTransactionsWireDto.
 */
export function parsePaginatedTransactionsWireDto(raw: unknown): PaginatedTransactionsWireDto {
  if (!isObject(raw)) {
    throw new ContractViolationError("PaginatedTransactionsWireDto must be an object", raw);
  }

  if (!Array.isArray(raw.items)) {
    throw new ContractViolationError("PaginatedTransactionsWireDto.items must be an array", raw);
  }

  const items = Object.freeze(raw.items.map(parseTransactionWireDto));

  const totalCount =
    typeof raw.totalCount === "number" && Number.isInteger(raw.totalCount) && raw.totalCount >= 0
      ? raw.totalCount
      : 0;

  const page =
    typeof raw.page === "number" && Number.isInteger(raw.page) && raw.page >= 1 ? raw.page : 1;

  const pageSize =
    typeof raw.pageSize === "number" && Number.isInteger(raw.pageSize) && raw.pageSize >= 1
      ? raw.pageSize
      : 20;

  const totalPages =
    typeof raw.totalPages === "number" && Number.isInteger(raw.totalPages) && raw.totalPages >= 1
      ? raw.totalPages
      : 1;

  return Object.freeze({
    items,
    totalCount,
    page,
    pageSize,
    totalPages,
  });
}

/**
 * Validates and decodes a raw JSON value into a strongly-typed Transaction domain entity.
 */
export function parseTransaction(raw: unknown): Transaction {
  if (!isObject(raw)) {
    throw new ContractViolationError("Transaction payload must be an object", raw);
  }

  const id = toTransactionId(raw.id);

  const source: TransactionSource =
    raw.source === "import" ? "import" : raw.source === "manual" ? "manual" : "manual";

  if (typeof raw.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(raw.date)) {
    throw new ContractViolationError(
      "Transaction.date must be an ISO date string (YYYY-MM-DD)",
      raw,
    );
  }

  const description = typeof raw.description === "string" ? raw.description : null;
  const payee = typeof raw.payee === "string" ? raw.payee : "";

  const amount = toMinorUnits(raw.amount);

  const typeId = toTypeId(raw.typeId);

  const type = typeof raw.type === "string" ? raw.type.trim() : "";

  const typeColor =
    typeof raw.typeColor === "string" && raw.typeColor.trim().length > 0
      ? raw.typeColor.trim()
      : "#71717a";

  const memberId =
    typeof raw.memberId === "number" && Number.isInteger(raw.memberId) && raw.memberId > 0
      ? (raw.memberId as MemberId)
      : (1 as MemberId);

  if (typeof raw.accountId !== "number" || !Number.isInteger(raw.accountId) || raw.accountId <= 0) {
    throw new ContractViolationError("Transaction.accountId must be a positive integer", raw);
  }

  const toAccountId =
    typeof raw.toAccountId === "number" && Number.isInteger(raw.toAccountId) && raw.toAccountId > 0
      ? (raw.toAccountId as AccountId)
      : undefined;

  if (
    typeof raw.categoryId !== "number" ||
    !Number.isInteger(raw.categoryId) ||
    raw.categoryId <= 0
  ) {
    throw new ContractViolationError("Transaction.categoryId must be a positive integer", raw);
  }

  const subcategoryId =
    typeof raw.subcategoryId === "number" &&
    Number.isInteger(raw.subcategoryId) &&
    raw.subcategoryId > 0
      ? (raw.subcategoryId as SubcategoryId)
      : undefined;

  const notes =
    typeof raw.notes === "string" && raw.notes.trim().length > 0 ? raw.notes.trim() : undefined;

  const status: TransactionStatus = raw.status === "pending" ? "pending" : "cleared";

  return Object.freeze({
    id,
    source,
    date: raw.date,
    description,
    payee,
    amount,
    typeId,
    type,
    typeColor,
    memberId,
    accountId: raw.accountId as AccountId,
    ...(toAccountId !== undefined ? { toAccountId } : {}),
    categoryId: raw.categoryId as CategoryId,
    ...(subcategoryId !== undefined ? { subcategoryId } : {}),
    ...(notes !== undefined ? { notes } : {}),
    status,
  });
}

/**
 * Validates and narrows an unknown payload to an immutable array of Transactions.
 */
export function parseTransactionsResponse(raw: unknown): readonly Transaction[] {
  if (!Array.isArray(raw)) {
    throw new ContractViolationError("Transactions response must be an array", raw);
  }
  return Object.freeze(raw.map(parseTransaction));
}

export interface TransactionQueryFilters {
  readonly query?: string;
  readonly fromDate?: string;
  readonly toDate?: string;
  readonly startDate?: string;
  readonly endDate?: string;
  readonly memberId?: number;
  readonly accountId?: number;
  readonly accountIds?: readonly number[];
  readonly categoryId?: number;
  readonly categoryIds?: readonly number[];
  readonly subcategoryId?: number;
  readonly subcategoryIds?: readonly number[];
  readonly typeId?: TypeId | number;
  readonly typeIds?: readonly number[];
  readonly type?: TransactionType;
  readonly status?: TransactionStatus;
  readonly statuses?: readonly TransactionStatus[];
  readonly minAmount?: number;
  readonly maxAmount?: number;
  readonly page?: number;
  readonly pageSize?: number;
}

/**
 * Single Source of Truth interface for Transaction API interactions.
 */
export interface TransactionsTransport {
  getTransactions(filters?: TransactionQueryFilters): Promise<PaginatedTransactionsWireDto>;
  getTransaction(id: TransactionId | string): Promise<TransactionWireDto>;
  createTransaction(payload: CreateTransactionInput): Promise<TransactionWireDto>;
  updateTransaction(
    id: TransactionId | string,
    payload: UpdateTransactionInput,
  ): Promise<TransactionWireDto>;
  deleteTransaction(id: TransactionId | string, source?: TransactionSource): Promise<null>;
}
