import { ContractViolationError, isObject } from "$lib/api";
import {
  toAccountId,
  toCategoryId,
  toMinorUnits,
  toSubcategoryId,
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

export { toAccountId, toCategoryId, toMinorUnits, toSubcategoryId, toTransactionId, toTypeId };
export type { AccountId, CategoryId, MemberId, MinorUnits, SubcategoryId, TransactionId, TypeId };

export type TransactionType = string;
export type TransactionStatus = "cleared" | "pending";
export type TransactionSource = "manual" | "import";

/**
 * Authoritative transaction domain entity matching the database table and backend TransactionDto.
 */
export interface Transaction {
  readonly id: TransactionId;
  readonly source: TransactionSource;
  readonly date: string; // ISO date 'YYYY-MM-DD'
  readonly description: string | null;
  readonly payee: string | null;
  readonly amount: MinorUnits;
  readonly typeId: TypeId;
  readonly accountId: AccountId;
  readonly categoryId: CategoryId;
  readonly subcategoryId?: SubcategoryId;
  readonly notes?: string;
  readonly status: TransactionStatus;
}

/**
 * Paginated DTO envelope returned by GET /api/v1/transactions.
 */
export interface PaginatedTransactionsDto {
  readonly items: readonly Transaction[];
  readonly totalCount: number;
  readonly page: number;
  readonly pageSize: number;
  readonly totalPages: number;
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
  const payee = typeof raw.payee === "string" ? raw.payee : null;

  const amount = toMinorUnits(raw.amount);
  const typeId = toTypeId(raw.typeId);

  if (typeof raw.accountId !== "number" || !Number.isInteger(raw.accountId) || raw.accountId <= 0) {
    throw new ContractViolationError("Transaction.accountId must be a positive integer", raw);
  }

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
    accountId: raw.accountId as AccountId,
    categoryId: raw.categoryId as CategoryId,
    ...(subcategoryId !== undefined ? { subcategoryId } : {}),
    ...(notes !== undefined ? { notes } : {}),
    status,
  });
}

/**
 * Validates and decodes raw JSON into PaginatedTransactionsDto.
 */
export function parsePaginatedTransactions(raw: unknown): PaginatedTransactionsDto {
  if (!isObject(raw)) {
    throw new ContractViolationError("PaginatedTransactionsDto must be an object", raw);
  }

  if (!Array.isArray(raw.items)) {
    throw new ContractViolationError("PaginatedTransactionsDto.items must be an array", raw);
  }

  const items = Object.freeze(raw.items.map(parseTransaction));

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
 * Validates and narrows an unknown payload to an immutable array of Transactions.
 */
export function parseTransactionsResponse(raw: unknown): readonly Transaction[] {
  if (!Array.isArray(raw)) {
    throw new ContractViolationError("Transactions response must be an array", raw);
  }
  return Object.freeze(raw.map(parseTransaction));
}

export type NewTransaction = Omit<Transaction, "id">;

export interface TransactionQueryFilters {
  readonly query?: string;
  readonly startDate?: string;
  readonly endDate?: string;
  readonly accountIds?: readonly AccountId[];
  readonly categoryIds?: readonly CategoryId[];
  readonly subcategoryIds?: readonly SubcategoryId[];
  readonly typeIds?: readonly TypeId[];
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
  getTransactions(filters?: TransactionQueryFilters): Promise<PaginatedTransactionsDto>;
  getTransaction(id: TransactionId): Promise<Transaction>;
  createTransaction(payload: NewTransaction): Promise<Transaction>;
  updateTransaction(transaction: Transaction): Promise<Transaction>;
  deleteTransaction(id: TransactionId, source?: TransactionSource): Promise<null>;
}
