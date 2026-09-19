import { ContractViolationError, isObject } from "$lib/api";

export interface Family {
  id: string;
  name: string;
  createdAt: string;
}

export interface Member {
  id: string;
  familyId: string;
  name: string;
  createdAt: string;
}

export type AccountType = "bank_account" | "credit_card";

export interface BaseAccount {
  id: string;
  familyId: string;
  ownerMemberId: string;
  type: AccountType;
  bankName: string;
  last4: string;
  createdAt: string;
}

export interface BankAccount extends BaseAccount {
  type: "bank_account";
}

export interface CreditCardAccount extends BaseAccount {
  type: "credit_card";
  cardName: string;
  creditLimit: number;
}

export type Account = BankAccount | CreditCardAccount;

export interface CreateMemberInput {
  name: string;
}

export interface UpdateMemberInput {
  id: string;
  name: string;
}

export interface CreateBankAccountInput {
  ownerMemberId: string;
  bankName: string;
  last4: string;
}

export interface UpdateBankAccountInput {
  id: string;
  bankName: string;
  last4: string;
}

export interface CreateCreditCardInput {
  ownerMemberId: string;
  bankName: string;
  cardName: string;
  last4: string;
  creditLimit: number;
}

export interface UpdateCreditCardInput {
  id: string;
  bankName: string;
  cardName: string;
  last4: string;
  creditLimit: number;
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Family.
 */
export function parseFamily(raw: unknown): Family {
  if (!isObject(raw)) {
    throw new ContractViolationError("Family payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("Family.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("Family.name must be a string", raw);
  }
  if (typeof raw.createdAt !== "string") {
    throw new ContractViolationError("Family.createdAt must be a string", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    createdAt: raw.createdAt,
  };
}

/**
 * Validates and narrows raw JSON data to a strongly-typed Member.
 */
export function parseMember(raw: unknown): Member {
  if (!isObject(raw)) {
    throw new ContractViolationError("Member payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("Member.id must be a string", raw);
  }
  if (typeof raw.familyId !== "string") {
    throw new ContractViolationError("Member.familyId must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("Member.name must be a string", raw);
  }
  if (typeof raw.createdAt !== "string") {
    throw new ContractViolationError("Member.createdAt must be a string", raw);
  }
  return {
    id: raw.id,
    familyId: raw.familyId,
    name: raw.name,
    createdAt: raw.createdAt,
  };
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
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("BankAccount.id must be a string", raw);
  }
  if (typeof raw.familyId !== "string") {
    throw new ContractViolationError("BankAccount.familyId must be a string", raw);
  }
  if (typeof raw.ownerMemberId !== "string") {
    throw new ContractViolationError("BankAccount.ownerMemberId must be a string", raw);
  }
  if (typeof raw.bankName !== "string") {
    throw new ContractViolationError("BankAccount.bankName must be a string", raw);
  }
  if (typeof raw.last4 !== "string") {
    throw new ContractViolationError("BankAccount.last4 must be a string", raw);
  }
  if (typeof raw.createdAt !== "string") {
    throw new ContractViolationError("BankAccount.createdAt must be a string", raw);
  }
  return {
    id: raw.id,
    familyId: raw.familyId,
    ownerMemberId: raw.ownerMemberId,
    type: "bank_account",
    bankName: raw.bankName,
    last4: raw.last4,
    createdAt: raw.createdAt,
  };
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
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("CreditCardAccount.id must be a string", raw);
  }
  if (typeof raw.familyId !== "string") {
    throw new ContractViolationError("CreditCardAccount.familyId must be a string", raw);
  }
  if (typeof raw.ownerMemberId !== "string") {
    throw new ContractViolationError("CreditCardAccount.ownerMemberId must be a string", raw);
  }
  if (typeof raw.bankName !== "string") {
    throw new ContractViolationError("CreditCardAccount.bankName must be a string", raw);
  }
  if (typeof raw.cardName !== "string") {
    throw new ContractViolationError("CreditCardAccount.cardName must be a string", raw);
  }
  if (typeof raw.last4 !== "string") {
    throw new ContractViolationError("CreditCardAccount.last4 must be a string", raw);
  }
  if (typeof raw.creditLimit !== "number" || Number.isNaN(raw.creditLimit)) {
    throw new ContractViolationError("CreditCardAccount.creditLimit must be a number", raw);
  }
  if (typeof raw.createdAt !== "string") {
    throw new ContractViolationError("CreditCardAccount.createdAt must be a string", raw);
  }
  return {
    id: raw.id,
    familyId: raw.familyId,
    ownerMemberId: raw.ownerMemberId,
    type: "credit_card",
    bankName: raw.bankName,
    cardName: raw.cardName,
    last4: raw.last4,
    creditLimit: raw.creditLimit,
    createdAt: raw.createdAt,
  };
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

export interface FamilyOverview {
  family: Family;
  members: Member[];
  accounts: Account[];
}

/**
 * Validates and narrows raw JSON data to a complete FamilyOverview payload.
 */
export function parseFamilyOverview(raw: unknown): FamilyOverview {
  if (!isObject(raw)) {
    throw new ContractViolationError("FamilyOverview payload must be an object", raw);
  }
  if (!Array.isArray(raw.members)) {
    throw new ContractViolationError("FamilyOverview.members must be an array", raw);
  }
  if (!Array.isArray(raw.accounts)) {
    throw new ContractViolationError("FamilyOverview.accounts must be an array", raw);
  }
  return {
    family: parseFamily(raw.family),
    members: raw.members.map(parseMember),
    accounts: raw.accounts.map(parseAccount),
  };
}
