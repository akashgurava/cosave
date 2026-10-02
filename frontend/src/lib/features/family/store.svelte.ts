import { MOCK_FAMILY, MOCK_MEMBERS, MOCK_ACCOUNTS, MOCK_SUPPORTED_CURRENCIES } from "./mock";
import { familyApi } from "./api";
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
  #isLoading = $state(false);
  #syncToBackend = typeof window !== "undefined";

  get isLoading(): boolean {
    return this.#isLoading;
  }

  setSyncToBackend(enabled: boolean): void {
    this.#syncToBackend = enabled;
  }

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
    if (this.#syncToBackend) {
      familyApi.updateFamily({ currency: code }).catch((err) => {
        console.warn("Failed to persist family base currency update to backend:", err);
      });
    }
  }

  async load(): Promise<void> {
    this.#isLoading = true;
    try {
      const details = await familyApi.getDetails();
      this.#family = details.family;
      this.#members = [...details.members];
      this.#accounts = [...details.accounts];
      if (
        this.#selectedMemberId === null ||
        !this.#members.some((m) => m.id === this.#selectedMemberId)
      ) {
        this.#selectedMemberId = this.#members[0]?.id ?? null;
      }
    } catch (err) {
      console.warn("Using local cached family state (backend offline or uninitialized):", err);
    } finally {
      this.#isLoading = false;
    }
  }

  async updateFamily(input: UpdateFamilyInput): Promise<Family> {
    if (this.#syncToBackend) {
      const updated = await familyApi.updateFamily(input);
      this.#family = updated;
      return updated;
    }
    this.#family = {
      ...this.#family,
      family_name:
        input.family_name !== undefined ? input.family_name.trim() : this.#family.family_name,
      currency: input.currency !== undefined ? input.currency : this.#family.currency,
    };
    return this.#family;
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

  async addMember(
    inputOrName: string | CreateMemberInput | { member_name: string },
  ): Promise<Member> {
    const rawName = typeof inputOrName === "string" ? inputOrName : inputOrName.member_name;
    const trimmed = rawName.trim();
    let newMember: Member;
    if (this.#syncToBackend) {
      newMember = await familyApi.createMember({
        family_id: this.#family.id,
        member_name: trimmed,
      });
    } else {
      newMember = {
        id: Date.now(),
        family_id: this.#family.id,
        member_name: trimmed,
        created_at: Math.floor(Date.now() / 1000),
      };
    }
    this.#members.push(newMember);
    if (this.#selectedMemberId === null) {
      this.#selectedMemberId = newMember.id;
    }
    return newMember;
  }

  async updateMember(
    idOrInput: number | { id: number; member_name: string },
    maybeName?: string,
  ): Promise<Member> {
    const id = typeof idOrInput === "number" ? idOrInput : idOrInput.id;
    const rawName = typeof idOrInput === "number" ? (maybeName ?? "") : idOrInput.member_name;
    const trimmed = rawName.trim();
    let updated: Member;
    if (this.#syncToBackend) {
      updated = await familyApi.updateMember(id, { member_name: trimmed });
    } else {
      const existing = this.#members.find((m) => m.id === id);
      if (!existing) {
        throw new Error(`Member with id ${id} not found`);
      }
      updated = {
        ...existing,
        member_name: trimmed,
      };
    }
    const idx = this.#members.findIndex((m) => m.id === id);
    if (idx !== -1) {
      this.#members[idx] = updated;
    }
    return updated;
  }

  async deleteMember(id: number): Promise<void> {
    if (this.#syncToBackend) {
      await familyApi.deleteMember(id);
    }
    this.#members = this.#members.filter((m: Member) => m.id !== id);
    this.#accounts = this.#accounts.filter((a: Account) => a.owner_member_id !== id);
    if (this.#selectedMemberId === id) {
      this.#selectedMemberId = this.#members[0]?.id ?? null;
    }
  }

  async addBankAccount(
    input: Omit<CreateBankAccountInput, "family_id" | "currency"> & {
      family_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    const familyId = input.family_id ?? this.#family.id;
    const fullPayload: CreateBankAccountInput = {
      family_id: familyId,
      owner_member_id: input.owner_member_id,
      currency: input.currency ?? this.#family.currency,
      bank_name: input.bank_name.trim(),
      account_name: input.account_name.trim(),
      last4: input.last4.trim(),
      available_balance_cents: input.available_balance_cents,
    };
    let newAcc: BankAccount;
    if (this.#syncToBackend) {
      newAcc = await familyApi.createBankAccount(fullPayload);
    } else {
      newAcc = {
        id: Date.now(),
        family_id: familyId,
        owner_member_id: fullPayload.owner_member_id,
        type: "bank_account",
        currency: fullPayload.currency,
        bank_name: fullPayload.bank_name,
        account_name: fullPayload.account_name,
        last4: fullPayload.last4,
        available_balance_cents: fullPayload.available_balance_cents,
        created_at: Math.floor(Date.now() / 1000),
      };
    }
    this.#accounts.push(newAcc);
    return newAcc;
  }

  async updateBankAccount(
    idOrInput: number | (UpdateBankAccountInput & { id: number }),
    maybeInput?: UpdateBankAccountInput,
  ): Promise<BankAccount> {
    const id = typeof idOrInput === "number" ? idOrInput : idOrInput.id;
    const input = typeof idOrInput === "number" ? maybeInput! : idOrInput;
    const payload: UpdateBankAccountInput = {
      currency: input.currency,
      bank_name: input.bank_name.trim(),
      account_name: input.account_name.trim(),
      last4: input.last4.trim(),
      available_balance_cents: input.available_balance_cents,
    };
    let updated: BankAccount;
    if (this.#syncToBackend) {
      updated = await familyApi.updateBankAccount(id, payload);
    } else {
      const existing = this.#accounts.find(
        (a: Account): a is BankAccount => a.id === id && a.type === "bank_account",
      );
      if (!existing) {
        throw new Error(`Bank account with id ${id} not found`);
      }
      updated = {
        ...existing,
        currency: payload.currency ?? existing.currency,
        bank_name: payload.bank_name,
        account_name: payload.account_name,
        last4: payload.last4,
        available_balance_cents: payload.available_balance_cents,
      };
    }
    const idx = this.#accounts.findIndex((a) => a.id === id);
    if (idx !== -1) {
      this.#accounts[idx] = updated;
    }
    return updated;
  }

  async addCreditCard(
    input: Omit<CreateCreditCardInput, "family_id" | "currency"> & {
      family_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    const familyId = input.family_id ?? this.#family.id;
    const limitCents = Math.max(0, input.credit_limit_cents);
    const availCents = Math.max(0, input.available_cents);
    const fullPayload: CreateCreditCardInput = {
      family_id: familyId,
      owner_member_id: input.owner_member_id,
      currency: input.currency ?? this.#family.currency,
      bank_name: input.bank_name.trim(),
      card_name: input.card_name.trim(),
      last4: input.last4.trim(),
      credit_limit_cents: limitCents,
      available_cents: availCents,
    };
    let newCard: CreditCardAccount;
    if (this.#syncToBackend) {
      newCard = await familyApi.createCreditCard(fullPayload);
    } else {
      newCard = {
        id: Date.now(),
        family_id: familyId,
        owner_member_id: fullPayload.owner_member_id,
        type: "credit_card",
        currency: fullPayload.currency,
        bank_name: fullPayload.bank_name,
        card_name: fullPayload.card_name,
        last4: fullPayload.last4,
        credit_limit_cents: limitCents,
        available_cents: availCents,
        outstanding_cents: limitCents - availCents,
        created_at: Math.floor(Date.now() / 1000),
      };
    }
    this.#accounts.push(newCard);
    return newCard;
  }

  async updateCreditCard(
    idOrInput: number | (UpdateCreditCardInput & { id: number }),
    maybeInput?: UpdateCreditCardInput,
  ): Promise<CreditCardAccount> {
    const id = typeof idOrInput === "number" ? idOrInput : idOrInput.id;
    const input = typeof idOrInput === "number" ? maybeInput! : idOrInput;
    const limitCents = Math.max(0, input.credit_limit_cents);
    const availCents = Math.max(0, input.available_cents);
    const payload: UpdateCreditCardInput = {
      currency: input.currency,
      bank_name: input.bank_name.trim(),
      card_name: input.card_name.trim(),
      last4: input.last4.trim(),
      credit_limit_cents: limitCents,
      available_cents: availCents,
    };
    let updated: CreditCardAccount;
    if (this.#syncToBackend) {
      updated = await familyApi.updateCreditCard(id, payload);
    } else {
      const existing = this.#accounts.find(
        (a: Account): a is CreditCardAccount => a.id === id && a.type === "credit_card",
      );
      if (!existing) {
        throw new Error(`Credit card with id ${id} not found`);
      }
      updated = {
        ...existing,
        currency: payload.currency ?? existing.currency,
        bank_name: payload.bank_name,
        card_name: payload.card_name,
        last4: payload.last4,
        credit_limit_cents: limitCents,
        available_cents: availCents,
        outstanding_cents: limitCents - availCents,
      };
    }
    const idx = this.#accounts.findIndex((a) => a.id === id);
    if (idx !== -1) {
      this.#accounts[idx] = updated;
    }
    return updated;
  }

  async deleteAccount(id: number): Promise<void> {
    if (this.#syncToBackend) {
      await familyApi.deleteAccount(id);
    }
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
