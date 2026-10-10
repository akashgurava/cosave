/**
 * Family domain entities, branded identifiers, and runtime schema contract decoders.
 *
 * Implements "Parse, Don't Validate" decoders for family composition, member rosters,
 * currencies, and financial accounts (`BankAccount`, `CreditCardAccount`). Enforces nominal
 * branding (`FamilyId`, `MemberId`, `AccountId`, `MinorUnits`) and freezes models.
 */

import { ContractViolationError, isObject } from "$lib/api";
import {
  toAccountId,
  toCurrencyId,
  toFamilyId,
  toMemberId,
  toMinorUnits,
  type AccountId,
  type CurrencyId,
  type Currency,
  type FamilyId,
  type MemberId,
  type MinorUnits,
} from "$lib/types";

export { toAccountId, toCurrencyId, toFamilyId, toMemberId, toMinorUnits };
export type { AccountId, Currency, CurrencyId, FamilyId, MemberId, MinorUnits };

export type CurrencyCode = string;

export interface CurrencyOption extends Currency {
  readonly id: CurrencyId;
  readonly code: CurrencyCode;
  readonly name: string;
  readonly symbol: string;
  readonly scale: number;
  readonly sortOrder?: number;
}

/**
 * Validates and narrows raw JSON data to a strongly-typed CurrencyOption.
 */
export function parseCurrencyOption(raw: unknown): CurrencyOption {
  if (isObject(raw) === false) {
    throw new ContractViolationError("CurrencyOption payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("CurrencyOption.id must be an integer", raw);
  }
  if (typeof raw.code !== "string" || raw.code.trim().length === 0) {
    throw new ContractViolationError("CurrencyOption.code must be a non-empty string", raw);
  }
  if (typeof raw.name !== "string" || raw.name.trim().length === 0) {
    throw new ContractViolationError("CurrencyOption.name must be a non-empty string", raw);
  }
  if (typeof raw.symbol !== "string" || raw.symbol.trim().length === 0) {
    throw new ContractViolationError("CurrencyOption.symbol must be a non-empty string", raw);
  }
  if (typeof raw.scale !== "number" || Number.isInteger(raw.scale) === false || raw.scale < 0) {
    throw new ContractViolationError("CurrencyOption.scale must be a non-negative integer", raw);
  }
  const sortOrder =
    typeof raw.sortOrder === "number" && Number.isInteger(raw.sortOrder) === true
      ? raw.sortOrder
      : undefined;

  return Object.freeze({
    id: raw.id as CurrencyId,
    code: raw.code.trim().toUpperCase(),
    name: raw.name.trim(),
    symbol: raw.symbol.trim(),
    scale: raw.scale,
    ...(sortOrder !== undefined ? { sortOrder } : {}),
  });
}

/**
 * Validates and narrows raw JSON array to a list of CurrencyOptions.
 */
export function parseCurrenciesResponse(raw: unknown): readonly CurrencyOption[] {
  if (Array.isArray(raw) === false) {
    throw new ContractViolationError("Currencies payload must be an array", raw);
  }
  return Object.freeze(raw.map(parseCurrencyOption));
}

/**
 * Validates default currency response payload.
 */
export function parseDefaultCurrencyResponse(raw: unknown): { readonly currency: CurrencyCode } {
  if (
    isObject(raw) === false ||
    typeof raw.currency !== "string" ||
    raw.currency.trim().length === 0
  ) {
    throw new ContractViolationError(
      "DefaultCurrency payload must be an object with non-empty currency string",
      raw,
    );
  }
  return Object.freeze({
    currency: raw.currency.trim().toUpperCase(),
  });
}

export interface Family {
  readonly id: FamilyId;
  readonly familyName: string;
  readonly currencyId: CurrencyId;
  readonly createdAt: number;
}

export interface Member {
  readonly id: MemberId;
  readonly familyId: FamilyId;
  readonly memberName: string;
  readonly createdAt: number;
}

export type AccountType = "bank_account" | "credit_card";

export interface BaseAccount {
  readonly id: AccountId;
  readonly familyId: FamilyId;
  readonly ownerMemberId: MemberId;
  readonly type: AccountType;
  readonly currencyId: CurrencyId;
  readonly bankName: string;
  readonly last4: string;
  readonly createdAt: number;
}

export interface BankAccount extends BaseAccount {
  readonly type: "bank_account";
  readonly accountName: string;
  readonly availableBalance: MinorUnits;
}

export interface CreditCardAccount extends BaseAccount {
  readonly type: "credit_card";
  readonly cardName: string;
  readonly creditLimit: MinorUnits;
  readonly availableCredit: MinorUnits;
  readonly outstandingBalance: MinorUnits;
}

export type Account = BankAccount | CreditCardAccount;

export interface CreateFamilyInput {
  readonly familyName: string;
  readonly currencyId: CurrencyId;
}

export interface UpdateFamilyInput {
  readonly familyId: FamilyId;
  readonly familyName: string;
}

export interface CreateMemberInput {
  readonly familyId: FamilyId;
  readonly memberName: string;
}

export interface UpdateMemberInput {
  readonly memberName: string;
}

export interface CreateBankAccountInput {
  readonly familyId: FamilyId;
  readonly ownerMemberId: MemberId;
  readonly currencyId?: CurrencyId;
  readonly bankName: string;
  readonly accountName: string;
  readonly last4: string;
  readonly availableBalance: MinorUnits;
}

export interface UpdateBankAccountInput {
  readonly currencyId?: CurrencyId;
  readonly bankName: string;
  readonly accountName: string;
  readonly last4: string;
  readonly availableBalance: MinorUnits;
}

export interface CreateCreditCardInput {
  readonly familyId: FamilyId;
  readonly ownerMemberId: MemberId;
  readonly currencyId?: CurrencyId;
  readonly bankName: string;
  readonly cardName: string;
  readonly last4: string;
  readonly creditLimit: MinorUnits;
  readonly availableCredit: MinorUnits;
}

export interface UpdateCreditCardInput {
  readonly currencyId?: CurrencyId;
  readonly bankName: string;
  readonly cardName: string;
  readonly last4: string;
  readonly creditLimit: MinorUnits;
  readonly availableCredit: MinorUnits;
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Family.
 */
export function parseFamily(raw: unknown): Family {
  if (isObject(raw) === false) {
    throw new ContractViolationError("Family payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("Family.id must be an integer", raw);
  }
  if (typeof raw.familyName !== "string" || raw.familyName.trim().length === 0) {
    throw new ContractViolationError("Family.familyName must be a non-empty string", raw);
  }
  if (typeof raw.currencyId !== "number" || Number.isInteger(raw.currencyId) === false) {
    throw new ContractViolationError("Family.currencyId must be an integer", raw);
  }
  if (typeof raw.createdAt !== "number" || Number.isInteger(raw.createdAt) === false) {
    throw new ContractViolationError("Family.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id as FamilyId,
    familyName: raw.familyName.trim(),
    currencyId: raw.currencyId as CurrencyId,
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Member.
 */
export function parseMember(raw: unknown): Member {
  if (isObject(raw) === false) {
    throw new ContractViolationError("Member payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("Member.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || Number.isInteger(raw.familyId) === false) {
    throw new ContractViolationError("Member.familyId must be an integer", raw);
  }
  if (typeof raw.memberName !== "string" || raw.memberName.trim().length === 0) {
    throw new ContractViolationError("Member.memberName must be a non-empty string", raw);
  }
  if (typeof raw.createdAt !== "number" || Number.isInteger(raw.createdAt) === false) {
    throw new ContractViolationError("Member.createdAt must be an epoch integer", raw);
  }
  return Object.freeze({
    id: raw.id as MemberId,
    familyId: raw.familyId as FamilyId,
    memberName: raw.memberName.trim(),
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a BankAccount.
 */
export function parseBankAccount(raw: unknown): BankAccount {
  if (isObject(raw) === false) {
    throw new ContractViolationError("BankAccount payload must be an object", raw);
  }
  if (raw.type !== "bank_account") {
    throw new ContractViolationError("BankAccount.type must be 'bank_account'", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("BankAccount.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || Number.isInteger(raw.familyId) === false) {
    throw new ContractViolationError("BankAccount.familyId must be an integer", raw);
  }
  if (typeof raw.ownerMemberId !== "number" || Number.isInteger(raw.ownerMemberId) === false) {
    throw new ContractViolationError("BankAccount.ownerMemberId must be an integer", raw);
  }
  if (typeof raw.currencyId !== "number" || Number.isInteger(raw.currencyId) === false) {
    throw new ContractViolationError("BankAccount.currencyId must be an integer", raw);
  }
  if (typeof raw.bankName !== "string" || raw.bankName.length === 0) {
    throw new ContractViolationError("BankAccount.bankName must be a non-empty string", raw);
  }
  if (typeof raw.accountName !== "string" || raw.accountName.length === 0) {
    throw new ContractViolationError("BankAccount.accountName must be a non-empty string", raw);
  }
  if (typeof raw.last4 !== "string" || raw.last4.length !== 4) {
    throw new ContractViolationError("BankAccount.last4 must be a 4-character string", raw);
  }
  if (
    typeof raw.availableBalance !== "number" ||
    Number.isInteger(raw.availableBalance) === false
  ) {
    throw new ContractViolationError("BankAccount.availableBalance must be an integer", raw);
  }
  if (typeof raw.createdAt !== "number" || Number.isInteger(raw.createdAt) === false) {
    throw new ContractViolationError("BankAccount.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id as AccountId,
    familyId: raw.familyId as FamilyId,
    ownerMemberId: raw.ownerMemberId as MemberId,
    type: "bank_account" as const,
    currencyId: raw.currencyId as CurrencyId,
    bankName: raw.bankName,
    accountName: raw.accountName,
    last4: raw.last4,
    availableBalance: toMinorUnits(raw.availableBalance),
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a CreditCardAccount.
 */
export function parseCreditCardAccount(raw: unknown): CreditCardAccount {
  if (isObject(raw) === false) {
    throw new ContractViolationError("CreditCardAccount payload must be an object", raw);
  }
  if (raw.type !== "credit_card") {
    throw new ContractViolationError("CreditCardAccount.type must be 'credit_card'", raw);
  }
  if (typeof raw.id !== "number" || Number.isInteger(raw.id) === false) {
    throw new ContractViolationError("CreditCardAccount.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || Number.isInteger(raw.familyId) === false) {
    throw new ContractViolationError("CreditCardAccount.familyId must be an integer", raw);
  }
  if (typeof raw.ownerMemberId !== "number" || Number.isInteger(raw.ownerMemberId) === false) {
    throw new ContractViolationError("CreditCardAccount.ownerMemberId must be an integer", raw);
  }
  if (typeof raw.currencyId !== "number" || Number.isInteger(raw.currencyId) === false) {
    throw new ContractViolationError("CreditCardAccount.currencyId must be an integer", raw);
  }
  if (typeof raw.bankName !== "string" || raw.bankName.length === 0) {
    throw new ContractViolationError("CreditCardAccount.bankName must be a non-empty string", raw);
  }
  if (typeof raw.cardName !== "string" || raw.cardName.length === 0) {
    throw new ContractViolationError("CreditCardAccount.cardName must be a non-empty string", raw);
  }
  if (typeof raw.last4 !== "string" || raw.last4.length !== 4) {
    throw new ContractViolationError("CreditCardAccount.last4 must be a 4-character string", raw);
  }
  if (
    typeof raw.creditLimit !== "number" ||
    Number.isInteger(raw.creditLimit) === false ||
    raw.creditLimit < 0
  ) {
    throw new ContractViolationError(
      "CreditCardAccount.creditLimit must be a non-negative integer",
      raw,
    );
  }
  if (typeof raw.availableCredit !== "number" || Number.isInteger(raw.availableCredit) === false) {
    throw new ContractViolationError("CreditCardAccount.availableCredit must be an integer", raw);
  }
  if (
    typeof raw.outstandingBalance !== "number" ||
    Number.isInteger(raw.outstandingBalance) === false
  ) {
    throw new ContractViolationError(
      "CreditCardAccount.outstandingBalance must be an integer",
      raw,
    );
  }
  if (typeof raw.createdAt !== "number" || Number.isInteger(raw.createdAt) === false) {
    throw new ContractViolationError("CreditCardAccount.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id as AccountId,
    familyId: raw.familyId as FamilyId,
    ownerMemberId: raw.ownerMemberId as MemberId,
    type: "credit_card" as const,
    currencyId: raw.currencyId as CurrencyId,
    bankName: raw.bankName,
    cardName: raw.cardName,
    last4: raw.last4,
    creditLimit: toMinorUnits(raw.creditLimit),
    availableCredit: toMinorUnits(raw.availableCredit),
    outstandingBalance: toMinorUnits(raw.outstandingBalance),
    createdAt: raw.createdAt,
  });
}

/**
 * Discriminated union parser for Account (Rust-grade tagged enum).
 */
export function parseAccount(raw: unknown): Account {
  if (isObject(raw) === false) {
    throw new ContractViolationError("Account payload must be an object", raw);
  }
  if (raw.type === "bank_account") {
    return parseBankAccount(raw);
  }
  if (raw.type === "credit_card") {
    return parseCreditCardAccount(raw);
  }
  throw new ContractViolationError(
    `Unknown account type discriminator: ${JSON.stringify(raw.type)}`,
    raw,
  );
}

export interface FamilyDetails {
  readonly family: Family | null;
  readonly members: readonly Member[];
  readonly accounts: readonly Account[];
  readonly currencies: readonly CurrencyOption[];
}

export type FamilyOverview = FamilyDetails;

/**
 * Validates and narrows raw JSON data to a complete FamilyDetails payload.
 */
export function parseFamilyDetails(raw: unknown): FamilyDetails {
  if (isObject(raw) === false) {
    throw new ContractViolationError("FamilyDetails payload must be an object", raw);
  }
  if (Array.isArray(raw.members) === false) {
    throw new ContractViolationError("FamilyDetails.members must be an array", raw);
  }
  if (Array.isArray(raw.accounts) === false) {
    throw new ContractViolationError("FamilyDetails.accounts must be an array", raw);
  }
  if (Array.isArray(raw.currencies) === false) {
    throw new ContractViolationError("FamilyDetails.currencies must be an array", raw);
  }
  const family = raw.family === null || raw.family === undefined ? null : parseFamily(raw.family);
  return Object.freeze({
    family,
    members: Object.freeze(raw.members.map(parseMember)),
    accounts: Object.freeze(raw.accounts.map(parseAccount)),
    currencies: Object.freeze(raw.currencies.map(parseCurrencyOption)),
  });
}

export const parseFamilyOverview = parseFamilyDetails;

/**
 * Transport contract defining all network operations supported by the family subsystem.
 * Enables dependency injection and Phase 1 UI testing against mock/in-memory transports.
 */
export interface FamilyTransport {
  getDetails(): Promise<FamilyDetails>;
  getOverview(): Promise<FamilyDetails>;
  getCurrencies(): Promise<readonly CurrencyOption[]>;
  createFamily(payload: CreateFamilyInput): Promise<Family>;
  updateFamily(payload: UpdateFamilyInput): Promise<Family>;
  getDefaultCurrency(region?: string): Promise<{ readonly currency: CurrencyCode }>;
  createMember(payload: CreateMemberInput): Promise<Member>;
  updateMember(id: MemberId, payload: UpdateMemberInput): Promise<Member>;
  deleteMember(id: MemberId): Promise<null>;
  createBankAccount(payload: CreateBankAccountInput): Promise<BankAccount>;
  updateBankAccount(id: AccountId, payload: UpdateBankAccountInput): Promise<BankAccount>;
  createCreditCard(payload: CreateCreditCardInput): Promise<CreditCardAccount>;
  updateCreditCard(id: AccountId, payload: UpdateCreditCardInput): Promise<CreditCardAccount>;
  deleteAccount(id: AccountId): Promise<null>;
}
