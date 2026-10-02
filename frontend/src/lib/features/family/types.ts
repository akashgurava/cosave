import { ContractViolationError, isObject } from "$lib/api";

export type CurrencyCode = string;

export interface CurrencyOption {
  readonly code: CurrencyCode;
  readonly symbol: string;
  readonly name: string;
  readonly scale: number;
}

export interface Family {
  readonly id: number;
  readonly name: string;
  readonly currency: CurrencyCode;
  readonly created_at: number;
}

export interface Member {
  readonly id: number;
  readonly family_id: number;
  readonly name: string;
  readonly created_at: number;
}

export type AccountType = "bank_account" | "credit_card";

export interface BaseAccount {
  readonly id: number;
  readonly family_id: number;
  readonly owner_member_id: number;
  readonly type: AccountType;
  readonly currency: CurrencyCode;
  readonly bank_name: string;
  readonly last4: string;
  readonly created_at: number;
}

export interface BankAccount extends BaseAccount {
  readonly type: "bank_account";
  readonly account_name: string;
  readonly available_balance_cents: number;
}

export interface CreditCardAccount extends BaseAccount {
  readonly type: "credit_card";
  readonly card_name: string;
  readonly credit_limit_cents: number;
  readonly available_cents: number;
  readonly outstanding_cents: number;
}

export type Account = BankAccount | CreditCardAccount;

export interface UpdateFamilyInput {
  readonly name?: string;
  readonly currency?: CurrencyCode;
}

export interface CreateMemberInput {
  readonly family_id: number;
  readonly name: string;
}

export interface UpdateMemberInput {
  readonly name: string;
}

export interface CreateBankAccountInput {
  readonly family_id: number;
  readonly owner_member_id: number;
  readonly currency: CurrencyCode;
  readonly bank_name: string;
  readonly account_name: string;
  readonly last4: string;
  readonly available_balance_cents: number;
}

export interface UpdateBankAccountInput {
  readonly currency?: CurrencyCode;
  readonly bank_name: string;
  readonly account_name: string;
  readonly last4: string;
  readonly available_balance_cents: number;
}

export interface CreateCreditCardInput {
  readonly family_id: number;
  readonly owner_member_id: number;
  readonly currency: CurrencyCode;
  readonly bank_name: string;
  readonly card_name: string;
  readonly last4: string;
  readonly credit_limit_cents: number;
  readonly available_cents: number;
}

export interface UpdateCreditCardInput {
  readonly currency?: CurrencyCode;
  readonly bank_name: string;
  readonly card_name: string;
  readonly last4: string;
  readonly credit_limit_cents: number;
  readonly available_cents: number;
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
  if (typeof raw.name !== "string" || raw.name.length === 0) {
    throw new ContractViolationError("Family.name must be a non-empty string", raw);
  }
  if (typeof raw.created_at !== "number" || !Number.isInteger(raw.created_at)) {
    throw new ContractViolationError("Family.created_at must be an epoch integer", raw);
  }
  if (typeof raw.currency !== "string" || raw.currency.trim().length === 0) {
    throw new ContractViolationError("Family.currency must be a non-empty string", raw);
  }
  const currency: CurrencyCode = raw.currency.trim().toUpperCase();

  return Object.freeze({
    id: raw.id,
    name: raw.name,
    currency,
    created_at: raw.created_at,
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
  if (typeof raw.family_id !== "number" || !Number.isInteger(raw.family_id)) {
    throw new ContractViolationError("Member.family_id must be an integer", raw);
  }
  if (typeof raw.name !== "string" || raw.name.length === 0) {
    throw new ContractViolationError("Member.name must be a non-empty string", raw);
  }
  if (typeof raw.created_at !== "number" || !Number.isInteger(raw.created_at)) {
    throw new ContractViolationError("Member.created_at must be an epoch integer", raw);
  }
  return Object.freeze({
    id: raw.id,
    family_id: raw.family_id,
    name: raw.name,
    created_at: raw.created_at,
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
  if (typeof raw.family_id !== "number" || !Number.isInteger(raw.family_id)) {
    throw new ContractViolationError("BankAccount.family_id must be an integer", raw);
  }
  if (typeof raw.owner_member_id !== "number" || !Number.isInteger(raw.owner_member_id)) {
    throw new ContractViolationError("BankAccount.owner_member_id must be an integer", raw);
  }
  if (typeof raw.bank_name !== "string" || raw.bank_name.length === 0) {
    throw new ContractViolationError("BankAccount.bank_name must be a non-empty string", raw);
  }
  if (typeof raw.account_name !== "string" || raw.account_name.length === 0) {
    throw new ContractViolationError("BankAccount.account_name must be a non-empty string", raw);
  }
  if (typeof raw.last4 !== "string" || raw.last4.length !== 4) {
    throw new ContractViolationError("BankAccount.last4 must be a 4-character string", raw);
  }
  if (
    typeof raw.available_balance_cents !== "number" ||
    !Number.isInteger(raw.available_balance_cents)
  ) {
    throw new ContractViolationError("BankAccount.available_balance_cents must be an integer", raw);
  }
  if (typeof raw.created_at !== "number" || !Number.isInteger(raw.created_at)) {
    throw new ContractViolationError("BankAccount.created_at must be an epoch integer", raw);
  }
  if (typeof raw.currency !== "string" || raw.currency.trim().length === 0) {
    throw new ContractViolationError("BankAccount.currency must be a non-empty string", raw);
  }
  const currency: CurrencyCode = raw.currency.trim().toUpperCase();

  return Object.freeze({
    id: raw.id,
    family_id: raw.family_id,
    owner_member_id: raw.owner_member_id,
    type: "bank_account" as const,
    currency,
    bank_name: raw.bank_name,
    account_name: raw.account_name,
    last4: raw.last4,
    available_balance_cents: raw.available_balance_cents,
    created_at: raw.created_at,
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
  if (typeof raw.family_id !== "number" || !Number.isInteger(raw.family_id)) {
    throw new ContractViolationError("CreditCardAccount.family_id must be an integer", raw);
  }
  if (typeof raw.owner_member_id !== "number" || !Number.isInteger(raw.owner_member_id)) {
    throw new ContractViolationError("CreditCardAccount.owner_member_id must be an integer", raw);
  }
  if (typeof raw.bank_name !== "string" || raw.bank_name.length === 0) {
    throw new ContractViolationError("CreditCardAccount.bank_name must be a non-empty string", raw);
  }
  if (typeof raw.card_name !== "string" || raw.card_name.length === 0) {
    throw new ContractViolationError("CreditCardAccount.card_name must be a non-empty string", raw);
  }
  if (typeof raw.last4 !== "string" || raw.last4.length !== 4) {
    throw new ContractViolationError("CreditCardAccount.last4 must be a 4-character string", raw);
  }
  if (
    typeof raw.credit_limit_cents !== "number" ||
    !Number.isInteger(raw.credit_limit_cents) ||
    raw.credit_limit_cents < 0
  ) {
    throw new ContractViolationError(
      "CreditCardAccount.credit_limit_cents must be a non-negative integer",
      raw,
    );
  }
  if (typeof raw.available_cents !== "number" || !Number.isInteger(raw.available_cents)) {
    throw new ContractViolationError("CreditCardAccount.available_cents must be an integer", raw);
  }
  if (typeof raw.created_at !== "number" || !Number.isInteger(raw.created_at)) {
    throw new ContractViolationError("CreditCardAccount.created_at must be an epoch integer", raw);
  }

  if (typeof raw.currency !== "string" || raw.currency.trim().length === 0) {
    throw new ContractViolationError("CreditCardAccount.currency must be a non-empty string", raw);
  }
  const currency: CurrencyCode = raw.currency.trim().toUpperCase();

  const outstanding_cents =
    typeof raw.outstanding_cents === "number" && Number.isInteger(raw.outstanding_cents)
      ? raw.outstanding_cents
      : raw.credit_limit_cents - raw.available_cents;

  return Object.freeze({
    id: raw.id,
    family_id: raw.family_id,
    owner_member_id: raw.owner_member_id,
    type: "credit_card" as const,
    currency,
    bank_name: raw.bank_name,
    card_name: raw.card_name,
    last4: raw.last4,
    credit_limit_cents: raw.credit_limit_cents,
    available_cents: raw.available_cents,
    outstanding_cents,
    created_at: raw.created_at,
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
  readonly family: Family;
  readonly members: readonly Member[];
  readonly accounts: readonly Account[];
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
  return Object.freeze({
    family: parseFamily(raw.family),
    members: Object.freeze(raw.members.map(parseMember)),
    accounts: Object.freeze(raw.accounts.map(parseAccount)),
  });
}

export const parseFamilyOverview = parseFamilyDetails;
