import { api, parseNull } from "$lib/api";
import {
  parseTransaction,
  parseTransactionsResponse,
  type CreateTransactionInput,
  type Transaction,
  type TransactionId,
  type TransactionQueryFilters,
  type TransactionsTransport,
  type UpdateTransactionInput,
} from "./types";

/**
 * Transactions API client with runtime schema contract enforcement.
 * Mirrors the authoritative Rust Axum backend Single Source of Truth (SSOT).
 */
export const transactionsApi: TransactionsTransport = {
  getTransactions(filters?: TransactionQueryFilters): Promise<readonly Transaction[]> {
    const query: Record<string, string | number> = {};
    if (filters?.query !== undefined && filters.query.trim().length > 0) {
      query.query = filters.query.trim();
    }
    if (filters?.fromDate !== undefined) query.from_date = filters.fromDate;
    if (filters?.toDate !== undefined) query.to_date = filters.toDate;
    if (filters?.memberId !== undefined) query.member_id = filters.memberId;
    if (filters?.accountId !== undefined) query.account_id = filters.accountId;
    if (filters?.categoryId !== undefined) query.category_id = filters.categoryId;
    if (filters?.type !== undefined) query.type = filters.type;
    if (filters?.status !== undefined) query.status = filters.status;
    if (filters?.minAmount !== undefined) query.min_amount = filters.minAmount;
    if (filters?.maxAmount !== undefined) query.max_amount = filters.maxAmount;

    return api.get<readonly Transaction[]>("/api/v1/transactions", {
      query: Object.keys(query).length > 0 ? query : undefined,
      schema: parseTransactionsResponse,
    });
  },

  getTransaction(id: TransactionId | number): Promise<Transaction> {
    return api.get<Transaction>("/api/v1/transactions/:id", {
      pathParams: { id },
      schema: parseTransaction,
    });
  },

  createTransaction(payload: CreateTransactionInput): Promise<Transaction> {
    return api.post<Transaction>("/api/v1/transactions", payload, {
      schema: parseTransaction,
    });
  },

  updateTransaction(
    id: TransactionId | number,
    payload: UpdateTransactionInput,
  ): Promise<Transaction> {
    return api.patch<Transaction>("/api/v1/transactions/:id", payload, {
      pathParams: { id },
      schema: parseTransaction,
    });
  },

  deleteTransaction(id: TransactionId | number): Promise<null> {
    return api.delete<null>("/api/v1/transactions/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },
};
