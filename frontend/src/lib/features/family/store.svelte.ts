import { MOCK_FAMILY, MOCK_MEMBERS, MOCK_ACCOUNTS, MOCK_SUPPORTED_CURRENCIES } from "./mock";
import type {
  Family,
  Member,
  Account,
  BankAccount,
  CreditCardAccount,
  CurrencyCode,
  CurrencyOption,
  UpdateFamilyInput,
  CreateMemberInput,
  UpdateMemberInput,
  CreateBankAccountInput,
  UpdateBankAccountInput,
  CreateCreditCardInput,
  UpdateCreditCardInput,
} from "./types";

class FamilyStore {
  #family = $state<Family>({ ...MOCK_FAMILY });
  #members = $state<Member[]>([...MOCK_MEMBERS]);
  #accounts = $state<Account[]>([...MOCK_ACCOUNTS]);
  #currencies = $state<readonly CurrencyOption[]>([...MOCK_SUPPORTED_CURRENCIES]);
  #selectedMemberId = $state<number | null>(MOCK_MEMBERS[0]?.id ?? null);

  get currencies(): readonly CurrencyOption[] {
    return this.#currencies;
  }

  get family(): Family {
    return this.#family;
  }

  get currency(): CurrencyCode {
    return this.#family.currency;
  }

  set currency(code: CurrencyCode) {
    this.#family = {
      ...this.#family,
      currency: code,
    };
  }

  updateFamily(input: UpdateFamilyInput): boolean {
    this.#family = {
      ...this.#family,
      name: input.name !== undefined ? input.name.trim() : this.#family.name,
      currency: input.currency !== undefined ? input.currency : this.#family.currency,
    };
    return true;
  }

  get members(): Member[] {
    return this.#members;
  }

  get accounts(): Account[] {
    return this.#accounts;
  }

  get selectedMemberId(): number | null {
    return this.#selectedMemberId;
  }

  set selectedMemberId(id: number | null) {
    this.#selectedMemberId = id;
  }

  // Computed aggregates
  totalCreditLimitCents: number = $derived(
    this.#accounts
      .filter((a): a is CreditCardAccount => a.type === "credit_card")
      .reduce((sum: number, a: CreditCardAccount) => sum + a.credit_limit_cents, 0),
  );

  totalCreditLimit: number = $derived(Math.round(this.totalCreditLimitCents / 100));

  totalAvailableCreditCents: number = $derived(
    this.#accounts
      .filter((a): a is CreditCardAccount => a.type === "credit_card")
      .reduce((sum: number, a: CreditCardAccount) => sum + a.available_cents, 0),
  );

  totalAvailableCredit: number = $derived(Math.round(this.totalAvailableCreditCents / 100));

  totalOutstandingCreditCents: number = $derived(
    this.#accounts
      .filter((a): a is CreditCardAccount => a.type === "credit_card")
      .reduce((sum: number, a: CreditCardAccount) => sum + a.outstanding_cents, 0),
  );

  totalOutstandingCredit: number = $derived(Math.round(this.totalOutstandingCreditCents / 100));

  totalBankBalanceCents: number = $derived(
    this.#accounts
      .filter((a): a is BankAccount => a.type === "bank_account")
      .reduce((sum: number, a: BankAccount) => sum + a.available_balance_cents, 0),
  );

  totalBankBalance: number = $derived(Math.round(this.totalBankBalanceCents / 100));

  totalBankAccounts: number = $derived(
    this.#accounts.filter((a: Account) => a.type === "bank_account").length,
  );

  totalCreditCards: number = $derived(
    this.#accounts.filter((a: Account) => a.type === "credit_card").length,
  );

  getMember(id: number | null | undefined): Member | null {
    if (id === null || id === undefined) {
      return null;
    }
    return this.#members.find((m: Member) => m.id === id) ?? null;
  }

  getMemberAccounts(memberId: number): Account[] {
    return this.#accounts.filter((a: Account) => a.owner_member_id === memberId);
  }

  getMemberBankAccounts(memberId: number): BankAccount[] {
    return this.#accounts.filter(
      (a: Account): a is BankAccount => a.owner_member_id === memberId && a.type === "bank_account",
    );
  }

  getMemberCreditCards(memberId: number): CreditCardAccount[] {
    return this.#accounts.filter(
      (a: Account): a is CreditCardAccount =>
        a.owner_member_id === memberId && a.type === "credit_card",
    );
  }

  getMemberCreditLimitCents(memberId: number): number {
    return this.getMemberCreditCards(memberId).reduce(
      (sum: number, c: CreditCardAccount) => sum + c.credit_limit_cents,
      0,
    );
  }

  getMemberCreditLimit(memberId: number): number {
    return Math.round(this.getMemberCreditLimitCents(memberId) / 100);
  }

  getMemberAvailableCreditCents(memberId: number): number {
    return this.getMemberCreditCards(memberId).reduce(
      (sum: number, c: CreditCardAccount) => sum + c.available_cents,
      0,
    );
  }

  getMemberOutstandingCreditCents(memberId: number): number {
    return this.getMemberCreditCards(memberId).reduce(
      (sum: number, c: CreditCardAccount) => sum + c.outstanding_cents,
      0,
    );
  }

  getMemberBankBalanceCents(memberId: number): number {
    return this.getMemberBankAccounts(memberId).reduce(
      (sum: number, b: BankAccount) => sum + b.available_balance_cents,
      0,
    );
  }

  addMember(input: CreateMemberInput): Member {
    const newId = Date.now();
    const newMember: Member = {
      id: newId,
      family_id: this.#family.id,
      name: input.name.trim(),
      created_at: Math.floor(Date.now() / 1000),
    };
    this.#members.push(newMember);
    if (this.#selectedMemberId === null) {
      this.#selectedMemberId = newMember.id;
    }
    return newMember;
  }

  updateMember(input: UpdateMemberInput): boolean {
    const m = this.#members.find((item: Member) => item.id === input.id);
    if (!m) {
      return false;
    }
    (m as { name: string }).name = input.name.trim();
    return true;
  }

  deleteMember(id: number): void {
    this.#members = this.#members.filter((m: Member) => m.id !== id);
    this.#accounts = this.#accounts.filter((a: Account) => a.owner_member_id !== id);
    if (this.#selectedMemberId === id) {
      this.#selectedMemberId = this.#members[0]?.id ?? null;
    }
  }

  addBankAccount(input: CreateBankAccountInput): BankAccount {
    const newAcc: BankAccount = {
      id: Date.now(),
      family_id: this.#family.id,
      owner_member_id: input.owner_member_id,
      type: "bank_account",
      currency: input.currency ?? this.#family.currency,
      bank_name: input.bank_name.trim(),
      account_name: input.account_name.trim(),
      last4: input.last4.trim(),
      available_balance_cents: input.available_balance_cents,
      created_at: Math.floor(Date.now() / 1000),
    };
    this.#accounts.push(newAcc);
    return newAcc;
  }

  updateBankAccount(input: UpdateBankAccountInput): boolean {
    const acc = this.#accounts.find(
      (a: Account): a is BankAccount => a.id === input.id && a.type === "bank_account",
    );
    if (!acc) {
      return false;
    }
    const mutableAcc = acc as {
      currency: CurrencyCode;
      bank_name: string;
      account_name: string;
      last4: string;
      available_balance_cents: number;
    };
    if (input.currency !== undefined) {
      mutableAcc.currency = input.currency;
    }
    mutableAcc.bank_name = input.bank_name.trim();
    mutableAcc.account_name = input.account_name.trim();
    mutableAcc.last4 = input.last4.trim();
    mutableAcc.available_balance_cents = input.available_balance_cents;
    return true;
  }

  addCreditCard(input: CreateCreditCardInput): CreditCardAccount {
    const limitCents = Math.max(0, input.credit_limit_cents);
    const availCents = Math.max(0, input.available_cents);
    const newCard: CreditCardAccount = {
      id: Date.now(),
      family_id: this.#family.id,
      owner_member_id: input.owner_member_id,
      type: "credit_card",
      currency: input.currency ?? this.#family.currency,
      bank_name: input.bank_name.trim(),
      card_name: input.card_name.trim(),
      last4: input.last4.trim(),
      credit_limit_cents: limitCents,
      available_cents: availCents,
      outstanding_cents: limitCents - availCents,
      created_at: Math.floor(Date.now() / 1000),
    };
    this.#accounts.push(newCard);
    return newCard;
  }

  updateCreditCard(input: UpdateCreditCardInput): boolean {
    const card = this.#accounts.find(
      (a: Account): a is CreditCardAccount => a.id === input.id && a.type === "credit_card",
    );
    if (!card) {
      return false;
    }
    const mutableCard = card as {
      currency: CurrencyCode;
      bank_name: string;
      card_name: string;
      last4: string;
      credit_limit_cents: number;
      available_cents: number;
      outstanding_cents: number;
    };
    const limitCents = Math.max(0, input.credit_limit_cents);
    const availCents = Math.max(0, input.available_cents);
    if (input.currency !== undefined) {
      mutableCard.currency = input.currency;
    }
    mutableCard.bank_name = input.bank_name.trim();
    mutableCard.card_name = input.card_name.trim();
    mutableCard.last4 = input.last4.trim();
    mutableCard.credit_limit_cents = limitCents;
    mutableCard.available_cents = availCents;
    mutableCard.outstanding_cents = limitCents - availCents;
    return true;
  }

  deleteAccount(id: number): void {
    this.#accounts = this.#accounts.filter((a: Account) => a.id !== id);
  }

  resetToDefaults(): void {
    this.#family = { ...MOCK_FAMILY };
    this.#members = [...MOCK_MEMBERS];
    this.#accounts = [...MOCK_ACCOUNTS];
    this.#currencies = [...MOCK_SUPPORTED_CURRENCIES];
    this.#selectedMemberId = MOCK_MEMBERS[0]?.id ?? null;
  }
}

export const familyStore = new FamilyStore();
