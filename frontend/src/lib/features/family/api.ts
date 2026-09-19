import { api } from "$lib/api";
import {
  parseBankAccount,
  parseCreditCardAccount,
  parseFamilyOverview,
  parseMember,
  type BankAccount,
  type CreateBankAccountInput,
  type CreateCreditCardInput,
  type CreateMemberInput,
  type CreditCardAccount,
  type FamilyOverview,
  type Member,
  type UpdateBankAccountInput,
  type UpdateCreditCardInput,
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

  createMember(payload: CreateMemberInput): Promise<Member> {
    return api.post<Member>("/api/v1/family/members", payload, {
      schema: parseMember,
    });
  },

  updateMember(payload: UpdateMemberInput): Promise<Member> {
    return api.put<Member>("/api/v1/family/members/:id", payload, {
      pathParams: { id: payload.id },
      schema: parseMember,
    });
  },

  deleteMember(id: string): Promise<void> {
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
    return api.put<BankAccount>("/api/v1/family/accounts/bank/:id", payload, {
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
    return api.put<CreditCardAccount>("/api/v1/family/accounts/credit/:id", payload, {
      pathParams: { id: payload.id },
      schema: parseCreditCardAccount,
    });
  },

  deleteAccount(id: string): Promise<void> {
    return api.delete<void>("/api/v1/family/accounts/:id", {
      pathParams: { id },
    });
  },
};
