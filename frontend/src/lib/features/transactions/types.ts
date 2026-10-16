import { ContractViolationError, isObject } from "$lib/api";
import {
  AccountId,
  CategoryId,
  createEnum,
  MemberId,
  MinorUnits,
  SubcategoryId,
  TransactionId,
  TypeId,
} from "$lib/types";

export { AccountId, CategoryId, MemberId, MinorUnits, SubcategoryId, TransactionId, TypeId };

export type TransactionType = string;

/**
 * Settlement status of a transaction (cleared or pending).
 */
export const TransactionStatus = createEnum("TransactionStatus", {
  Cleared: "cleared",
  Pending: "pending",
});
export type TransactionStatus = (typeof TransactionStatus.values)[number];

/**
 * Source of a transaction (manual entry or statement import).
 */
export const TransactionSource = createEnum("TransactionSource", {
  Manual: "manual",
  Import: "import",
});
export type TransactionSource = (typeof TransactionSource.values)[number];

/**
 * Transaction record matching the master ledger.
 */
export interface Transaction {
  readonly id: TransactionId;
  readonly source: TransactionSource;
  readonly accountId: AccountId;
  readonly typeId: TypeId;
  readonly categoryId: CategoryId;
  readonly subcategoryId?: SubcategoryId;
  readonly amount: MinorUnits;
  readonly date: string; // ISO date 'YYYY-MM-DD'
  readonly description: string | null;
  readonly payee: string | null;
  readonly notes?: string;
  readonly status: TransactionStatus;
}

/**
 * Paginated list response for transactions.
 */
export interface PaginatedTransactionsDto {
  readonly items: readonly Transaction[];
  readonly totalCount: number;
  readonly page: number;
  readonly pageSize: number;
  readonly totalPages: number;
}

/**
 * Date range presets for filtering transactions.
 */
export const DatePreset = createEnum("DatePreset", {
  All: "all",
  D1: "1d",
  D3: "3d",
  D7: "7d",
  M1: "1m",
  M3: "3m",
  M6: "6m",
  Y1: "1y",
  Custom: "custom",
});
export type DatePreset = (typeof DatePreset.values)[number];

export const AMOUNT_FIXED_POINTS = [0, 100, 500, 1000, 2000, Infinity] as const;
export const AMOUNT_POINT_LABELS = ["$0", "$100", "$500", "$1k", "$2k", "> $2k"] as const;

/**
 * Amount preset for filtering transactions.
 */
export const AmountPreset = createEnum("AmountPreset", {
  All: "all",
  Custom: "custom",
});
export type AmountPreset = (typeof AmountPreset.values)[number];

/**
 * Column to sort transactions by.
 */
export const SortField = createEnum("SortField", {
  Date: "date",
  Description: "description",
  Payee: "payee",
  Amount: "amount",
  Type: "type",
  Member: "member",
  Account: "account",
  Category: "category",
  Status: "status",
});
export type SortField = (typeof SortField.values)[number];

/**
 * Sort order (ascending or descending).
 */
export const SortDirection = createEnum("SortDirection", {
  Asc: "asc",
  Desc: "desc",
});
export type SortDirection = (typeof SortDirection.values)[number];

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

  const id = TransactionId(raw.id);

  const source = TransactionSource.parse(raw.source);

  if (typeof raw.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(raw.date)) {
    throw new ContractViolationError(
      "Transaction.date must be an ISO date string (YYYY-MM-DD)",
      raw,
    );
  }

  const description = typeof raw.description === "string" ? raw.description : null;
  const payee = typeof raw.payee === "string" ? raw.payee : null;

  const amount = MinorUnits(raw.amount);
  const typeId = TypeId(raw.typeId);
  const accountId = AccountId(raw.accountId);
  const categoryId = CategoryId(raw.categoryId);

  const subcategoryId =
    raw.subcategoryId !== null && raw.subcategoryId !== undefined
      ? SubcategoryId(raw.subcategoryId)
      : undefined;

  const notes =
    typeof raw.notes === "string" && raw.notes.trim().length > 0 ? raw.notes.trim() : undefined;

  const status = TransactionStatus.parse(raw.status);

  return Object.freeze({
    id,
    source,
    accountId,
    typeId,
    categoryId,
    ...(subcategoryId !== undefined ? { subcategoryId } : {}),
    amount,
    date: raw.date,
    description,
    payee,
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
  deleteTransaction(id: TransactionId, source: TransactionSource): Promise<null>;
}
