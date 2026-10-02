import { api, parseNull } from "$lib/api";
import {
  parseBankAccount,
  parseCreditCardAccount,
  parseCurrenciesResponse,
  parseDefaultCurrencyResponse,
  parseFamily,
  parseFamilyDetails,
  parseMember,
  type BankAccount,
  type CreateBankAccountInput,
  type CreateCreditCardInput,
  type CreateMemberInput,
  type CreditCardAccount,
  type CurrencyCode,
  type CurrencyOption,
  type Family,
  type FamilyDetails,
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
  getDetails(): Promise<FamilyDetails> {
    return api.get<FamilyDetails>("/api/v1/config/family", {
      schema: parseFamilyDetails,
    });
  },

  getOverview(): Promise<FamilyDetails> {
    return this.getDetails();
  },

  getCurrencies(): Promise<readonly CurrencyOption[]> {
    return api.get<readonly CurrencyOption[]>("/api/v1/config/currencies", {
      schema: parseCurrenciesResponse,
    });
  },

  updateFamily(payload: UpdateFamilyInput): Promise<Family> {
    return api.patch<Family>("/api/v1/config/family", payload, {
      schema: parseFamily,
    });
  },

  getDefaultCurrency(region?: string): Promise<{ readonly currency: CurrencyCode }> {
    return api.get<{ readonly currency: CurrencyCode }>("/api/v1/config/currency/default", {
      query: region ? { region } : undefined,
      schema: parseDefaultCurrencyResponse,
    });
  },

  createMember(payload: CreateMemberInput): Promise<Member> {
    return api.post<Member>("/api/v1/config/members", payload, {
      schema: parseMember,
    });
  },

  updateMember(id: number, payload: UpdateMemberInput): Promise<Member> {
    return api.patch<Member>("/api/v1/config/members/:id", payload, {
      pathParams: { id },
      schema: parseMember,
    });
  },

  deleteMember(id: number): Promise<null> {
    return api.delete<null>("/api/v1/config/members/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },

  createBankAccount(payload: CreateBankAccountInput): Promise<BankAccount> {
    return api.post<BankAccount>("/api/v1/config/accounts/bank", payload, {
      schema: parseBankAccount,
    });
  },

  updateBankAccount(id: number, payload: UpdateBankAccountInput): Promise<BankAccount> {
    return api.patch<BankAccount>("/api/v1/config/accounts/bank/:id", payload, {
      pathParams: { id },
      schema: parseBankAccount,
    });
  },

  createCreditCard(payload: CreateCreditCardInput): Promise<CreditCardAccount> {
    return api.post<CreditCardAccount>("/api/v1/config/accounts/credit", payload, {
      schema: parseCreditCardAccount,
    });
  },

  updateCreditCard(id: number, payload: UpdateCreditCardInput): Promise<CreditCardAccount> {
    return api.patch<CreditCardAccount>("/api/v1/config/accounts/credit/:id", payload, {
      pathParams: { id },
      schema: parseCreditCardAccount,
    });
  },

  deleteAccount(id: number): Promise<null> {
    return api.delete<null>("/api/v1/config/accounts/:id", {
      pathParams: { id },
      schema: parseNull,
    });
  },
};
