import { ContractViolationError, isObject } from "$lib/api";

export type CurrencyCode = string;

export interface CurrencyOption {
  readonly id: number;
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
  if (!isObject(raw)) {
    throw new ContractViolationError("CurrencyOption payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || !Number.isInteger(raw.id)) {
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
  if (typeof raw.scale !== "number" || !Number.isInteger(raw.scale) || raw.scale < 0) {
    throw new ContractViolationError("CurrencyOption.scale must be a non-negative integer", raw);
  }
  const sortOrder =
    typeof raw.sortOrder === "number" && Number.isInteger(raw.sortOrder)
      ? raw.sortOrder
      : undefined;

  return Object.freeze({
    id: raw.id,
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
  if (!Array.isArray(raw)) {
    throw new ContractViolationError("Currencies payload must be an array", raw);
  }
  return Object.freeze(raw.map(parseCurrencyOption));
}

/**
 * Validates default currency response payload.
 */
export function parseDefaultCurrencyResponse(raw: unknown): { readonly currency: CurrencyCode } {
  if (!isObject(raw) || typeof raw.currency !== "string" || raw.currency.trim().length === 0) {
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
  readonly id: number;
  readonly familyName: string;
  readonly currencyId: number;
  readonly createdAt: number;
}

export interface Member {
  readonly id: number;
  readonly familyId: number;
  readonly memberName: string;
  readonly createdAt: number;
}

export type AccountType = "bank_account" | "credit_card";

export interface BaseAccount {
  readonly id: number;
  readonly familyId: number;
  readonly ownerMemberId: number;
  readonly type: AccountType;
  readonly currencyId: number;
  readonly bankName: string;
  readonly last4: string;
  readonly createdAt: number;
}

export interface BankAccount extends BaseAccount {
  readonly type: "bank_account";
  readonly accountName: string;
  readonly availableBalanceCents: number;
}

export interface CreditCardAccount extends BaseAccount {
  readonly type: "credit_card";
  readonly cardName: string;
  readonly creditLimitCents: number;
  readonly availableCents: number;
  readonly outstandingCents: number;
}

export type Account = BankAccount | CreditCardAccount;

export interface UpdateFamilyInput {
  readonly familyName?: string;
  readonly currencyId: number;
}

export interface CreateMemberInput {
  readonly familyId: number;
  readonly memberName: string;
}

export interface UpdateMemberInput {
  readonly memberName: string;
}

export interface CreateBankAccountInput {
  readonly familyId: number;
  readonly ownerMemberId: number;
  readonly currencyId: number;
  readonly bankName: string;
  readonly accountName: string;
  readonly last4: string;
  readonly availableBalanceCents: number;
}

export interface UpdateBankAccountInput {
  readonly currencyId: number;
  readonly bankName: string;
  readonly accountName: string;
  readonly last4: string;
  readonly availableBalanceCents: number;
}

export interface CreateCreditCardInput {
  readonly familyId: number;
  readonly ownerMemberId: number;
  readonly currencyId: number;
  readonly bankName: string;
  readonly cardName: string;
  readonly last4: string;
  readonly creditLimitCents: number;
  readonly availableCents: number;
}

export interface UpdateCreditCardInput {
  readonly currencyId: number;
  readonly bankName: string;
  readonly cardName: string;
  readonly last4: string;
  readonly creditLimitCents: number;
  readonly availableCents: number;
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Family.
 */
export function parseFamily(raw: unknown): Family {
  if (!isObject(raw)) {
    throw new ContractViolationError("Family payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || !Number.isInteger(raw.id)) {
    throw new ContractViolationError("Family.id must be an integer", raw);
  }
  if (typeof raw.familyName !== "string" || raw.familyName.trim().length === 0) {
    throw new ContractViolationError("Family.familyName must be a non-empty string", raw);
  }
  if (typeof raw.currencyId !== "number" || !Number.isInteger(raw.currencyId)) {
    throw new ContractViolationError("Family.currencyId must be an integer", raw);
  }
  if (typeof raw.createdAt !== "number" || !Number.isInteger(raw.createdAt)) {
    throw new ContractViolationError("Family.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id,
    familyName: raw.familyName.trim(),
    currencyId: raw.currencyId,
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Member.
 */
export function parseMember(raw: unknown): Member {
  if (!isObject(raw)) {
    throw new ContractViolationError("Member payload must be an object", raw);
  }
  if (typeof raw.id !== "number" || !Number.isInteger(raw.id)) {
    throw new ContractViolationError("Member.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || !Number.isInteger(raw.familyId)) {
    throw new ContractViolationError("Member.familyId must be an integer", raw);
  }
  if (typeof raw.memberName !== "string" || raw.memberName.trim().length === 0) {
    throw new ContractViolationError("Member.memberName must be a non-empty string", raw);
  }
  if (typeof raw.createdAt !== "number" || !Number.isInteger(raw.createdAt)) {
    throw new ContractViolationError("Member.createdAt must be an epoch integer", raw);
  }
  return Object.freeze({
    id: raw.id,
    familyId: raw.familyId,
    memberName: raw.memberName.trim(),
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a BankAccount.
 */
export function parseBankAccount(raw: unknown): BankAccount {
  if (!isObject(raw)) {
    throw new ContractViolationError("BankAccount payload must be an object", raw);
  }
  if (raw.type !== "bank_account") {
    throw new ContractViolationError("BankAccount.type must be 'bank_account'", raw);
  }
  if (typeof raw.id !== "number" || !Number.isInteger(raw.id)) {
    throw new ContractViolationError("BankAccount.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || !Number.isInteger(raw.familyId)) {
    throw new ContractViolationError("BankAccount.familyId must be an integer", raw);
  }
  if (typeof raw.ownerMemberId !== "number" || !Number.isInteger(raw.ownerMemberId)) {
    throw new ContractViolationError("BankAccount.ownerMemberId must be an integer", raw);
  }
  if (typeof raw.currencyId !== "number" || !Number.isInteger(raw.currencyId)) {
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
    typeof raw.availableBalanceCents !== "number" ||
    !Number.isInteger(raw.availableBalanceCents)
  ) {
    throw new ContractViolationError("BankAccount.availableBalanceCents must be an integer", raw);
  }
  if (typeof raw.createdAt !== "number" || !Number.isInteger(raw.createdAt)) {
    throw new ContractViolationError("BankAccount.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id,
    familyId: raw.familyId,
    ownerMemberId: raw.ownerMemberId,
    type: "bank_account" as const,
    currencyId: raw.currencyId,
    bankName: raw.bankName,
    accountName: raw.accountName,
    last4: raw.last4,
    availableBalanceCents: raw.availableBalanceCents,
    createdAt: raw.createdAt,
  });
}

/**
 * Validates and narrows raw JSON data to a CreditCardAccount.
 */
export function parseCreditCardAccount(raw: unknown): CreditCardAccount {
  if (!isObject(raw)) {
    throw new ContractViolationError("CreditCardAccount payload must be an object", raw);
  }
  if (raw.type !== "credit_card") {
    throw new ContractViolationError("CreditCardAccount.type must be 'credit_card'", raw);
  }
  if (typeof raw.id !== "number" || !Number.isInteger(raw.id)) {
    throw new ContractViolationError("CreditCardAccount.id must be an integer", raw);
  }
  if (typeof raw.familyId !== "number" || !Number.isInteger(raw.familyId)) {
    throw new ContractViolationError("CreditCardAccount.familyId must be an integer", raw);
  }
  if (typeof raw.ownerMemberId !== "number" || !Number.isInteger(raw.ownerMemberId)) {
    throw new ContractViolationError("CreditCardAccount.ownerMemberId must be an integer", raw);
  }
  if (typeof raw.currencyId !== "number" || !Number.isInteger(raw.currencyId)) {
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
    typeof raw.creditLimitCents !== "number" ||
    !Number.isInteger(raw.creditLimitCents) ||
    raw.creditLimitCents < 0
  ) {
    throw new ContractViolationError(
      "CreditCardAccount.creditLimitCents must be a non-negative integer",
      raw,
    );
  }
  if (typeof raw.availableCents !== "number" || !Number.isInteger(raw.availableCents)) {
    throw new ContractViolationError("CreditCardAccount.availableCents must be an integer", raw);
  }
  if (typeof raw.outstandingCents !== "number" || !Number.isInteger(raw.outstandingCents)) {
    throw new ContractViolationError("CreditCardAccount.outstandingCents must be an integer", raw);
  }
  if (typeof raw.createdAt !== "number" || !Number.isInteger(raw.createdAt)) {
    throw new ContractViolationError("CreditCardAccount.createdAt must be an epoch integer", raw);
  }

  return Object.freeze({
    id: raw.id,
    familyId: raw.familyId,
    ownerMemberId: raw.ownerMemberId,
    type: "credit_card" as const,
    currencyId: raw.currencyId,
    bankName: raw.bankName,
    cardName: raw.cardName,
    last4: raw.last4,
    creditLimitCents: raw.creditLimitCents,
    availableCents: raw.availableCents,
    outstandingCents: raw.outstandingCents,
    createdAt: raw.createdAt,
  });
}

/**
 * Discriminated union parser for Account (Rust-grade tagged enum).
 */
export function parseAccount(raw: unknown): Account {
  if (!isObject(raw)) {
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
  if (!isObject(raw)) {
    throw new ContractViolationError("FamilyDetails payload must be an object", raw);
  }
  if (!Array.isArray(raw.members)) {
    throw new ContractViolationError("FamilyDetails.members must be an array", raw);
  }
  if (!Array.isArray(raw.accounts)) {
    throw new ContractViolationError("FamilyDetails.accounts must be an array", raw);
  }
  if (!Array.isArray(raw.currencies)) {
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
