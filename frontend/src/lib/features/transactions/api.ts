import { api, parseNull } from "$lib/api";
import {
  parsePaginatedTransactions,
  parseTransaction,
  type NewTransaction,
  type PaginatedTransactionsDto,
  type Transaction,
  type TransactionId,
  type TransactionQueryFilters,
  type TransactionSource,
  type TransactionsTransport,
} from "./types";

/**
 * Transactions API client with runtime schema contract enforcement.
 * Mirrors the authoritative Rust Axum backend Single Source of Truth (SSOT).
 */
export const transactionsApi: TransactionsTransport = {
  getTransactions(filters?: TransactionQueryFilters): Promise<PaginatedTransactionsDto> {
    const query: Record<string, string | number> = {};
    if (filters?.query !== undefined && filters.query.trim().length > 0) {
      query.q = filters.query.trim();
    }
    if (filters?.startDate !== undefined) query.startDate = filters.startDate;
    if (filters?.endDate !== undefined) query.endDate = filters.endDate;
    if (filters?.accountIds !== undefined && filters.accountIds.length > 0) {
      query.accountIds = filters.accountIds.join(",");
    }
    if (filters?.categoryIds !== undefined && filters.categoryIds.length > 0) {
      query.categoryIds = filters.categoryIds.join(",");
    }
    if (filters?.subcategoryIds !== undefined && filters.subcategoryIds.length > 0) {
      query.subcategoryIds = filters.subcategoryIds.join(",");
    }
    if (filters?.typeIds !== undefined && filters.typeIds.length > 0) {
      query.typeIds = filters.typeIds.map(Number).join(",");
    }
    if (filters?.statuses !== undefined && filters.statuses.length > 0) {
      query.statuses = filters.statuses.join(",");
    }
    if (filters?.minAmount !== undefined) query.minAmount = filters.minAmount;
    if (filters?.maxAmount !== undefined) query.maxAmount = filters.maxAmount;
    if (filters?.page !== undefined) query.page = filters.page;
    if (filters?.pageSize !== undefined) query.pageSize = filters.pageSize;

    return api.get<PaginatedTransactionsDto>("/api/v1/transactions", {
      query: Object.keys(query).length > 0 ? query : undefined,
      schema: parsePaginatedTransactions,
    });
  },

  getTransaction(id: TransactionId): Promise<Transaction> {
    return api.get<Transaction>("/api/v1/transactions/:id", {
      pathParams: { id },
      schema: parseTransaction,
    });
  },

  createTransaction(payload: NewTransaction): Promise<Transaction> {
    const { source: _source, ...wirePayload } = payload;

    return api.post<Transaction>("/api/v1/transactions", wirePayload, {
      schema: parseTransaction,
    });
  },

  updateTransaction(transaction: Transaction): Promise<Transaction> {
    const { id, ...wirePayload } = transaction;

    return api.patch<Transaction>("/api/v1/transactions/:id", wirePayload, {
      pathParams: { id },
      schema: parseTransaction,
    });
  },

  deleteTransaction(
    id: TransactionId,
    source: TransactionSource = "manual",
  ): Promise<null> {
    return api.delete<null>("/api/v1/transactions/:id", {
      pathParams: { id },
      body: { source },
      schema: parseNull,
    });
  },
};
