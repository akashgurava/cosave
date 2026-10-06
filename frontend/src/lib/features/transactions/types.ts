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

export interface Transaction {
  readonly id: TransactionId;
  readonly date: string; // ISO date 'YYYY-MM-DD'
  readonly description: string; // Statement description from bank/CC account
  readonly payee: string; // Counterparty (user-filled, mostly empty initially)
  readonly amount: MinorUnits;
  readonly typeId: TypeId;
  readonly type: string; // Dynamic type name from hierarchy (e.g. "Income", "Expense", "Transfer", "Invest")
  readonly typeColor: string; // Palette color hex from hierarchy (e.g. "#10b981", "#8b5cf6")
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

export interface CreateTransactionInput {
  readonly date: string;
  readonly description?: string;
  readonly payee?: string;
  readonly amount: MinorUnits;
  readonly typeId: TypeId;
  readonly type?: string;
  readonly typeColor?: string;
  readonly memberId: MemberId | number;
  readonly accountId: AccountId | number;
  readonly toAccountId?: AccountId | number;
  readonly categoryId: CategoryId | number;
  readonly subcategoryId?: SubcategoryId | number;
  readonly notes?: string;
  readonly status?: TransactionStatus;
}

export interface UpdateTransactionInput {
  readonly date?: string;
  readonly description?: string;
  readonly payee?: string;
  readonly amount?: MinorUnits;
  readonly typeId?: TypeId;
  readonly type?: string;
  readonly typeColor?: string;
  readonly memberId?: MemberId | number;
  readonly accountId?: AccountId | number;
  readonly toAccountId?: AccountId | number;
  readonly categoryId?: CategoryId | number;
  readonly subcategoryId?: SubcategoryId | number;
  readonly notes?: string;
  readonly status?: TransactionStatus;
}

/**
 * Validates and decodes a raw JSON value into a strongly-typed Transaction domain entity.
 */
export function parseTransaction(raw: unknown): Transaction {
  if (!isObject(raw)) {
    throw new ContractViolationError("Transaction payload must be an object", raw);
  }

  const id = toTransactionId(raw.id);

  if (typeof raw.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(raw.date)) {
    throw new ContractViolationError(
      "Transaction.date must be an ISO date string (YYYY-MM-DD)",
      raw,
    );
  }

  // Description is statement text from bank/CC; fall back to non-empty payee if missing in legacy inputs
  const rawDesc =
    typeof raw.description === "string" && raw.description.trim().length > 0
      ? raw.description.trim()
      : typeof raw.payee === "string" && raw.payee.trim().length > 0
        ? raw.payee.trim()
        : "";

  if (rawDesc.length === 0) {
    throw new ContractViolationError(
      "Transaction.description (or non-empty payee) must be provided",
      raw,
    );
  }

  const payee = typeof raw.payee === "string" ? raw.payee.trim() : "";

  const amount = toMinorUnits(raw.amount);

  const typeId = toTypeId(raw.typeId);

  if (typeof raw.type !== "string" || raw.type.trim().length === 0) {
    throw new ContractViolationError(
      "Transaction.type must be a non-empty string representing the category type",
      raw,
    );
  }
  const type = raw.type.trim();

  const typeColor =
    typeof raw.typeColor === "string" && raw.typeColor.trim().length > 0
      ? raw.typeColor.trim()
      : "#71717a";

  if (typeof raw.memberId !== "number" || !Number.isInteger(raw.memberId) || raw.memberId <= 0) {
    throw new ContractViolationError("Transaction.memberId must be a positive integer", raw);
  }

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
    date: raw.date,
    description: rawDesc,
    payee,
    amount,
    typeId,
    type,
    typeColor,
    memberId: raw.memberId as MemberId,
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
  readonly categoryId?: number;
  readonly typeId?: TypeId | number;
  readonly type?: TransactionType;
  readonly status?: TransactionStatus;
  readonly minAmount?: number;
  readonly maxAmount?: number;
}

/**
 * Single Source of Truth interface for Transaction API interactions.
 */
export interface TransactionsTransport {
  getTransactions(filters?: TransactionQueryFilters): Promise<readonly Transaction[]>;
  getTransaction(id: TransactionId | number): Promise<Transaction>;
  createTransaction(payload: CreateTransactionInput): Promise<Transaction>;
  updateTransaction(
    id: TransactionId | number,
    payload: UpdateTransactionInput,
  ): Promise<Transaction>;
  deleteTransaction(id: TransactionId | number): Promise<null>;
}
