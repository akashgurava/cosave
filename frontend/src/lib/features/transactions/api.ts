import { api, parseNull } from "$lib/api";
import {
  parsePaginatedTransactionsWireDto,
  parseTransactionWireDto,
  type CreateTransactionInput,
  type PaginatedTransactionsWireDto,
  type TransactionId,
  type TransactionQueryFilters,
  type TransactionSource,
  type TransactionsTransport,
  type TransactionWireDto,
  type UpdateTransactionInput,
} from "./types";

/**
 * Transactions API client with runtime schema contract enforcement.
 * Mirrors the authoritative Rust Axum backend Single Source of Truth (SSOT).
 */
export const transactionsApi: TransactionsTransport = {
  getTransactions(filters?: TransactionQueryFilters): Promise<PaginatedTransactionsWireDto> {
    const query: Record<string, string | number> = {};
    if (filters?.query !== undefined && filters.query.trim().length > 0) {
      query.q = filters.query.trim();
    }
    if (filters?.fromDate !== undefined) query.fromDate = filters.fromDate;
    if (filters?.toDate !== undefined) query.toDate = filters.toDate;
    if (filters?.startDate !== undefined) query.startDate = filters.startDate;
    if (filters?.endDate !== undefined) query.endDate = filters.endDate;
    if (filters?.memberId !== undefined) query.member_id = filters.memberId;
    if (filters?.accountId !== undefined) query.accountId = filters.accountId;
    if (filters?.accountIds !== undefined && filters.accountIds.length > 0) {
      query.accountIds = filters.accountIds.join(",");
    }
    if (filters?.categoryId !== undefined) query.categoryId = filters.categoryId;
    if (filters?.categoryIds !== undefined && filters.categoryIds.length > 0) {
      query.categoryIds = filters.categoryIds.join(",");
    }
    if (filters?.subcategoryId !== undefined) query.subcategoryId = filters.subcategoryId;
    if (filters?.subcategoryIds !== undefined && filters.subcategoryIds.length > 0) {
      query.subcategoryIds = filters.subcategoryIds.join(",");
    }
    if (filters?.typeId !== undefined) query.typeId = Number(filters.typeId);
    if (filters?.typeIds !== undefined && filters.typeIds.length > 0) {
      query.typeIds = filters.typeIds.map(Number).join(",");
    }
    if (filters?.status !== undefined) query.status = filters.status;
    if (filters?.statuses !== undefined && filters.statuses.length > 0) {
      query.statuses = filters.statuses.join(",");
    }
    if (filters?.minAmount !== undefined) query.minAmount = filters.minAmount;
    if (filters?.maxAmount !== undefined) query.maxAmount = filters.maxAmount;
    if (filters?.page !== undefined) query.page = filters.page;
    if (filters?.pageSize !== undefined) query.pageSize = filters.pageSize;

    return api.get<PaginatedTransactionsWireDto>("/api/v1/transactions", {
      query: Object.keys(query).length > 0 ? query : undefined,
      schema: parsePaginatedTransactionsWireDto,
    });
  },

  getTransaction(id: TransactionId | string): Promise<TransactionWireDto> {
    return api.get<TransactionWireDto>("/api/v1/transactions/:id", {
      pathParams: { id },
      schema: parseTransactionWireDto,
    });
  },

  createTransaction(payload: CreateTransactionInput): Promise<TransactionWireDto> {
    const wirePayload = {
      date: payload.date,
      ...(payload.description !== undefined ? { description: payload.description } : {}),
      ...(payload.payee !== undefined ? { payee: payload.payee } : {}),
      amount: payload.amount,
      typeId: Number(payload.typeId),
      accountId: Number(payload.accountId),
      categoryId: Number(payload.categoryId),
      ...(payload.subcategoryId !== undefined && payload.subcategoryId !== null
        ? { subcategoryId: Number(payload.subcategoryId) }
        : {}),
      ...(payload.notes !== undefined && payload.notes !== null ? { notes: payload.notes } : {}),
      ...(payload.status !== undefined ? { status: payload.status } : {}),
    };

    return api.post<TransactionWireDto>("/api/v1/transactions", wirePayload, {
      schema: parseTransactionWireDto,
    });
  },

  updateTransaction(
    id: TransactionId | string,
    payload: UpdateTransactionInput,
  ): Promise<TransactionWireDto> {
    const wirePayload = {
      source: payload.source,
      date: payload.date,
      ...(payload.description !== undefined ? { description: payload.description } : {}),
      ...(payload.payee !== undefined ? { payee: payload.payee } : {}),
      amount: payload.amount,
      typeId: Number(payload.typeId),
      accountId: Number(payload.accountId),
      categoryId: Number(payload.categoryId),
      ...(payload.subcategoryId !== undefined && payload.subcategoryId !== null
        ? { subcategoryId: Number(payload.subcategoryId) }
        : {}),
      ...(payload.notes !== undefined && payload.notes !== null ? { notes: payload.notes } : {}),
      ...(payload.status !== undefined ? { status: payload.status } : {}),
    };

    return api.patch<TransactionWireDto>("/api/v1/transactions/:id", wirePayload, {
      pathParams: { id },
      schema: parseTransactionWireDto,
    });
  },

  deleteTransaction(
    id: TransactionId | string,
    source: TransactionSource = "manual",
  ): Promise<null> {
    return api.delete<null>("/api/v1/transactions/:id", {
      pathParams: { id },
      body: { source },
      schema: parseNull,
    });
  },
};
