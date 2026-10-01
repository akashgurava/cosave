import { api } from "$lib/api";
import {
  parseBankAccount,
  parseCreditCardAccount,
  parseFamily,
  parseFamilyOverview,
  parseMember,
  type BankAccount,
  type CreateBankAccountInput,
  type CreateCreditCardInput,
  type CreateMemberInput,
  type CreditCardAccount,
  type CurrencyCode,
  type Family,
  type FamilyOverview,
  type Member,
  type UpdateBankAccountInput,
  type UpdateCreditCardInput,
  type UpdateFamilyInput,
  type UpdateMemberInput,
} from "./types";

/**
 * Family & Accounts API service functions with runtime schema contract enforcement.
 * The Rust backend is the authoritative Single Source of Truth (SSOT).
 */
export const familyApi = {
  getOverview(): Promise<FamilyOverview> {
    return api.get<FamilyOverview>("/api/v1/family", {
      schema: parseFamilyOverview,
    });
  },

  updateFamily(payload: UpdateFamilyInput): Promise<Family> {
    return api.patch<Family>("/api/v1/family", payload, {
      schema: parseFamily,
    });
  },

  getDefaultCurrency(region?: string): Promise<{ currency: CurrencyCode }> {
    return api.get<{ currency: CurrencyCode }>("/api/v1/family/currency/default", {
      query: region ? { region } : undefined,
    });
  },

  createMember(payload: CreateMemberInput): Promise<Member> {
    return api.post<Member>("/api/v1/family/members", payload, {
      schema: parseMember,
    });
  },

  updateMember(payload: UpdateMemberInput): Promise<Member> {
    return api.patch<Member>("/api/v1/family/members/:id", payload, {
      pathParams: { id: payload.id },
      schema: parseMember,
    });
  },

  deleteMember(id: number): Promise<void> {
    return api.delete<void>("/api/v1/family/members/:id", {
      pathParams: { id },
    });
  },

  createBankAccount(payload: CreateBankAccountInput): Promise<BankAccount> {
    return api.post<BankAccount>("/api/v1/family/accounts/bank", payload, {
      schema: parseBankAccount,
    });
  },

  updateBankAccount(payload: UpdateBankAccountInput): Promise<BankAccount> {
    return api.patch<BankAccount>("/api/v1/family/accounts/bank/:id", payload, {
      pathParams: { id: payload.id },
      schema: parseBankAccount,
    });
  },

  createCreditCard(payload: CreateCreditCardInput): Promise<CreditCardAccount> {
    return api.post<CreditCardAccount>("/api/v1/family/accounts/credit", payload, {
      schema: parseCreditCardAccount,
    });
  },

  updateCreditCard(payload: UpdateCreditCardInput): Promise<CreditCardAccount> {
    return api.patch<CreditCardAccount>("/api/v1/family/accounts/credit/:id", payload, {
      pathParams: { id: payload.id },
      schema: parseCreditCardAccount,
    });
  },

  deleteAccount(id: number): Promise<void> {
    return api.delete<void>("/api/v1/family/accounts/:id", {
      pathParams: { id },
    });
  },
};
