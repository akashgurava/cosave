import { familyApi } from "./api";
import { formatMoney, getBrowserRegion, getCurrencyScale, getCurrencySymbol } from "./currency";
import type {
  Account,
  BankAccount,
  CreateBankAccountInput,
  CreateCreditCardInput,
  CreateMemberInput,
  CreditCardAccount,
  CurrencyCode,
  CurrencyOption,
  Family,
  Member,
  UpdateBankAccountInput,
  UpdateCreditCardInput,
  UpdateFamilyInput,
} from "./types";

export class FamilyStore {
  #family = $state<Family | null>(null);
  #selectedCurrencyId = $state<number>(1);
  #members = $state<Member[]>([]);
  #accounts = $state<Account[]>([]);
  #currencies = $state<readonly CurrencyOption[]>([]);
  #selectedMemberId = $state<number | null>(null);
  #isLoading = $state(false);
  #isLoaded = $state(false);
  #error = $state<string | null>(null);

  get isLoading(): boolean {
    return this.#isLoading;
  }

  get isLoaded(): boolean {
    return this.#isLoaded;
  }

  get error(): string | null {
    return this.#error;
  }

  get currencies(): readonly CurrencyOption[] {
    return this.#currencies;
  }

  get family(): Family | null {
    return this.#family;
  }

  get currencyId(): number {
    return this.#family?.currency_id ?? this.#selectedCurrencyId;
  }

  get currency(): CurrencyCode {
    const id = this.currencyId;
    const found = this.#currencies.find((c) => c.id === id);
    return found?.code ?? "USD";
  }

  set currency(code: CurrencyCode) {
    const found = this.#currencies.find((c) => c.code === code);
    if (found !== undefined) {
      this.setCurrencyId(found.id).catch((err) => {
        console.warn("Failed to persist family base currency update to backend:", err);
      });
    }
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

  // Currency helpers backed by backend metadata
  getCurrencyOption(target?: number | CurrencyCode): CurrencyOption | undefined {
    if (typeof target === "number") {
      return this.#currencies.find((c) => c.id === target);
    }
    if (typeof target === "string") {
      return this.#currencies.find((c) => c.code === target);
    }
    return this.#currencies.find((c) => c.id === this.currencyId);
  }

  getCurrencySymbol(target?: number | CurrencyCode): string {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.symbol;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencySymbol(code);
  }

  getCurrencyScale(target?: number | CurrencyCode): number {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.scale;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencyScale(code);
  }

  formatMoney(amountCents: number, target?: number | CurrencyCode): string {
    const opt = this.getCurrencyOption(target);
    const code = opt?.code ?? (typeof target === "string" ? target : this.currency);
    return formatMoney(amountCents, code, opt);
  }

  async load(): Promise<void> {
    this.#isLoading = true;
    this.#error = null;
    try {
      const details = await familyApi.getDetails();
      this.#family = details.family;
      this.#members = [...details.members];
      this.#accounts = [...details.accounts];
      this.#currencies = [...details.currencies];

      if (details.family !== null) {
        this.#selectedCurrencyId = details.family.currency_id;
      } else {
        const defaultCurr = await familyApi.getDefaultCurrency(getBrowserRegion());
        const found = this.#currencies.find((c) => c.code === defaultCurr.currency);
        if (found !== undefined) {
          this.#selectedCurrencyId = found.id;
        }
      }

      if (
        this.#selectedMemberId === null ||
        !this.#members.some((m) => m.id === this.#selectedMemberId)
      ) {
        this.#selectedMemberId = this.#members[0]?.id ?? null;
      }
      this.#isLoaded = true;
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to load family configuration";
      this.#error = msg;
      console.warn("Failed to load family configuration from backend:", err);
    } finally {
      this.#isLoading = false;
    }
  }

  async setCurrencyId(id: number): Promise<Family> {
    this.#selectedCurrencyId = id;
    if (this.#family !== null) {
      this.#family = {
        ...this.#family,
        currency_id: id,
      };
    }
    const familyName = this.#family?.family_name ?? "My Family";
    const updated = await familyApi.updateFamily({
      family_name: familyName,
      currency_id: id,
    });
    this.#family = updated;
    return updated;
  }

  async updateFamily(input: Partial<UpdateFamilyInput>): Promise<Family> {
    const currencyId = input.currency_id ?? this.currencyId;
    const familyName = input.family_name ?? this.#family?.family_name ?? "My Family";
    const updated = await familyApi.updateFamily({
      family_name: familyName,
      currency_id: currencyId,
    });
    this.#family = updated;
    this.#selectedCurrencyId = updated.currency_id;
    return updated;
  }

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

  async addMember(
    inputOrName: string | CreateMemberInput | { member_name: string },
  ): Promise<Member> {
    const rawName = typeof inputOrName === "string" ? inputOrName : inputOrName.member_name;
    const familyId = this.#family?.id ?? 1;
    const newMember = await familyApi.createMember({
      family_id: familyId,
      member_name: rawName,
    });
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
    const updated = await familyApi.updateMember(id, { member_name: rawName });
    const idx = this.#members.findIndex((m) => m.id === id);
    if (idx !== -1) {
      this.#members[idx] = updated;
    }
    return updated;
  }

  async deleteMember(id: number): Promise<void> {
    await familyApi.deleteMember(id);
    this.#members = this.#members.filter((m: Member) => m.id !== id);
    this.#accounts = this.#accounts.filter((a: Account) => a.owner_member_id !== id);
    if (this.#selectedMemberId === id) {
      this.#selectedMemberId = this.#members[0]?.id ?? null;
    }
  }

  async addBankAccount(
    input: Omit<CreateBankAccountInput, "family_id" | "currency_id"> & {
      family_id?: number;
      currency_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    const familyId = input.family_id ?? this.#family?.id ?? 1;
    let currencyId = input.currency_id;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.#currencies.find((c) => c.code === input.currency)?.id;
    }
    const fullPayload: CreateBankAccountInput = {
      family_id: familyId,
      owner_member_id: input.owner_member_id,
      currency_id: currencyId ?? this.currencyId,
      bank_name: input.bank_name,
      account_name: input.account_name,
      last4: input.last4,
      available_balance_cents: input.available_balance_cents,
    };
    const newAcc = await familyApi.createBankAccount(fullPayload);
    this.#accounts.push(newAcc);
    return newAcc;
  }

  async updateBankAccount(
    idOrInput:
      | number
      | (Omit<UpdateBankAccountInput, "currency_id"> & {
          id: number;
          currency_id?: number;
          currency?: CurrencyCode;
        }),
    maybeInput?: Omit<UpdateBankAccountInput, "currency_id"> & {
      currency_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    const id = typeof idOrInput === "number" ? idOrInput : idOrInput.id;
    const input = typeof idOrInput === "number" ? maybeInput! : idOrInput;
    let currencyId = input.currency_id;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.#currencies.find((c) => c.code === input.currency)?.id;
    }
    const payload: UpdateBankAccountInput = {
      currency_id: currencyId ?? this.currencyId,
      bank_name: input.bank_name,
      account_name: input.account_name,
      last4: input.last4,
      available_balance_cents: input.available_balance_cents,
    };
    const updated = await familyApi.updateBankAccount(id, payload);
    const idx = this.#accounts.findIndex((a) => a.id === id);
    if (idx !== -1) {
      this.#accounts[idx] = updated;
    }
    return updated;
  }

  async addCreditCard(
    input: Omit<CreateCreditCardInput, "family_id" | "currency_id"> & {
      family_id?: number;
      currency_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    const familyId = input.family_id ?? this.#family?.id ?? 1;
    let currencyId = input.currency_id;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.#currencies.find((c) => c.code === input.currency)?.id;
    }
    const fullPayload: CreateCreditCardInput = {
      family_id: familyId,
      owner_member_id: input.owner_member_id,
      currency_id: currencyId ?? this.currencyId,
      bank_name: input.bank_name,
      card_name: input.card_name,
      last4: input.last4,
      credit_limit_cents: input.credit_limit_cents,
      available_cents: input.available_cents,
    };
    const newCard = await familyApi.createCreditCard(fullPayload);
    this.#accounts.push(newCard);
    return newCard;
  }

  async updateCreditCard(
    idOrInput:
      | number
      | (Omit<UpdateCreditCardInput, "currency_id"> & {
          id: number;
          currency_id?: number;
          currency?: CurrencyCode;
        }),
    maybeInput?: Omit<UpdateCreditCardInput, "currency_id"> & {
      currency_id?: number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    const id = typeof idOrInput === "number" ? idOrInput : idOrInput.id;
    const input = typeof idOrInput === "number" ? maybeInput! : idOrInput;
    let currencyId = input.currency_id;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.#currencies.find((c) => c.code === input.currency)?.id;
    }
    const payload: UpdateCreditCardInput = {
      currency_id: currencyId ?? this.currencyId,
      bank_name: input.bank_name,
      card_name: input.card_name,
      last4: input.last4,
      credit_limit_cents: input.credit_limit_cents,
      available_cents: input.available_cents,
    };
    const updated = await familyApi.updateCreditCard(id, payload);
    const idx = this.#accounts.findIndex((a) => a.id === id);
    if (idx !== -1) {
      this.#accounts[idx] = updated;
    }
    return updated;
  }

  async deleteAccount(id: number): Promise<void> {
    await familyApi.deleteAccount(id);
    this.#accounts = this.#accounts.filter((a: Account) => a.id !== id);
  }

  reset(): void {
    this.#family = null;
    this.#selectedCurrencyId = 1;
    this.#members = [];
    this.#accounts = [];
    this.#currencies = [];
    this.#selectedMemberId = null;
    this.#isLoading = false;
    this.#isLoaded = false;
    this.#error = null;
  }
}

export const familyStore = new FamilyStore();
