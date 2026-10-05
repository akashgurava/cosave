import { ApiError } from "$lib/api";
import type { AsyncState } from "$lib/types/core";
import { familyApi } from "./api";
import { formatMoney, getBrowserRegion, getCurrencyScale, getCurrencySymbol } from "./currency";
import type {
  Account,
  AccountId,
  AmountCents,
  BankAccount,
  CreateBankAccountInput,
  CreateCreditCardInput,
  CreateMemberInput,
  CreditCardAccount,
  CurrencyCode,
  CurrencyId,
  CurrencyOption,
  Family,
  FamilyDetails,
  FamilyId,
  FamilyTransport,
  Member,
  MemberId,
  UpdateBankAccountInput,
  UpdateCreditCardInput,
  UpdateFamilyInput,
} from "./types";

export class FamilyStore {
  #state = $state<AsyncState<FamilyDetails>>({ status: "idle" });
  #selectedCurrencyId = $state<CurrencyId | number>(1 as CurrencyId);
  #selectedMemberId = $state<MemberId | number | null>(null);
  #transport: FamilyTransport;

  constructor(transport: FamilyTransport = familyApi) {
    this.#transport = transport;
  }

  get state(): AsyncState<FamilyDetails> {
    return this.#state;
  }

  get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  get isLoaded(): boolean {
    return this.#state.status === "success";
  }

  get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  get currencies(): readonly CurrencyOption[] {
    return this.#state.status === "success" ? this.#state.data.currencies : [];
  }

  get family(): Family | null {
    return this.#state.status === "success" ? this.#state.data.family : null;
  }

  get currencyId(): CurrencyId | number {
    return this.family?.currencyId ?? this.#selectedCurrencyId;
  }

  get currency(): CurrencyCode {
    const id = this.currencyId;
    const found = this.currencies.find((c) => c.id === id);
    return found?.code ?? "USD";
  }

  set currency(code: CurrencyCode) {
    const found = this.currencies.find((c) => c.code === code);
    if (found !== undefined) {
      void this.setCurrencyId(found.id);
    }
  }

  get members(): readonly Member[] {
    return this.#state.status === "success" ? this.#state.data.members : [];
  }

  get accounts(): readonly Account[] {
    return this.#state.status === "success" ? this.#state.data.accounts : [];
  }

  get selectedMemberId(): MemberId | number | null {
    return this.#selectedMemberId;
  }

  set selectedMemberId(id: MemberId | number | null) {
    this.#selectedMemberId = id;
  }

  // Currency helpers backed by backend metadata
  getCurrencyOption(target?: CurrencyId | number | CurrencyCode): CurrencyOption | undefined {
    if (typeof target === "number") {
      return this.currencies.find((c) => c.id === target);
    }
    if (typeof target === "string") {
      return this.currencies.find((c) => c.code === target);
    }
    return this.currencies.find((c) => c.id === this.currencyId);
  }

  getCurrencySymbol(target?: CurrencyId | number | CurrencyCode): string {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.symbol;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencySymbol(code);
  }

  getCurrencyScale(target?: CurrencyId | number | CurrencyCode): number {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.scale;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencyScale(code);
  }

  formatMoney(
    amountCents: AmountCents | number,
    target?: CurrencyId | number | CurrencyCode,
  ): string {
    const opt = this.getCurrencyOption(target);
    const code = opt?.code ?? (typeof target === "string" ? target : this.currency);
    return formatMoney(amountCents, code, opt);
  }

  async load(): Promise<void> {
    this.#state = { status: "loading" };
    try {
      const details = await this.#transport.getDetails();
      if (details.family !== null) {
        this.#selectedCurrencyId = details.family.currencyId;
      } else {
        const defaultCurr = await this.#transport.getDefaultCurrency(getBrowserRegion());
        const found = details.currencies.find((c) => c.code === defaultCurr.currency);
        if (found !== undefined) {
          this.#selectedCurrencyId = found.id;
        }
      }

      if (
        this.#selectedMemberId === null ||
        !details.members.some((m) => m.id === this.#selectedMemberId)
      ) {
        this.#selectedMemberId = details.members[0]?.id ?? null;
      }
      this.#state = { status: "success", data: details };
    } catch (err) {
      const action =
        err instanceof ApiError
          ? (err.action ?? "CONFIG.FAMILY.LOAD.FAILED")
          : "CONFIG.FAMILY.LOAD.FAILED";
      const message = err instanceof Error ? err.message : "Failed to load family configuration";
      this.#state = { status: "error", error: { action, message } };
    }
  }

  async setCurrencyId(id: CurrencyId | number): Promise<Family> {
    this.#selectedCurrencyId = id;
    if (this.#state.status === "success" && this.#state.data.family !== null) {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          family: Object.freeze({
            ...this.#state.data.family,
            currencyId: id as CurrencyId,
          }),
        }),
      };
    }
    const familyName = this.family?.familyName ?? "My Family";
    const updated = await this.#transport.updateFamily({
      familyName,
      currencyId: id,
    });
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          family: updated,
        }),
      };
    }
    return updated;
  }

  async updateFamily(input: Partial<UpdateFamilyInput>): Promise<Family> {
    const currencyId = input.currencyId ?? this.currencyId;
    const familyName = input.familyName ?? this.family?.familyName ?? "My Family";
    const updated = await this.#transport.updateFamily({
      familyName,
      currencyId,
    });
    this.#selectedCurrencyId = updated.currencyId;
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          family: updated,
        }),
      };
    }
    return updated;
  }

  getMember(id: MemberId | number | null | undefined): Member | null {
    if (id === null || id === undefined) {
      return null;
    }
    return this.members.find((m: Member) => m.id === id) ?? null;
  }

  getMemberAccounts(memberId: MemberId | number): readonly Account[] {
    return this.accounts.filter((a: Account) => a.ownerMemberId === memberId);
  }

  getMemberBankAccounts(memberId: MemberId | number): readonly BankAccount[] {
    return this.accounts.filter(
      (a: Account): a is BankAccount => a.ownerMemberId === memberId && a.type === "bank_account",
    );
  }

  getMemberCreditCards(memberId: MemberId | number): readonly CreditCardAccount[] {
    return this.accounts.filter(
      (a: Account): a is CreditCardAccount =>
        a.ownerMemberId === memberId && a.type === "credit_card",
    );
  }

  async addMember(
    inputOrName: string | CreateMemberInput | { memberName: string },
  ): Promise<Member> {
    const rawName = typeof inputOrName === "string" ? inputOrName : inputOrName.memberName;
    const familyId = (this.family?.id ?? 1) as FamilyId;
    const newMember = await this.#transport.createMember({
      familyId,
      memberName: rawName,
    });
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          members: Object.freeze([...this.#state.data.members, newMember]),
        }),
      };
    }
    if (this.#selectedMemberId === null) {
      this.#selectedMemberId = newMember.id;
    }
    return newMember;
  }

  async updateMember(
    idOrInput: MemberId | number | { id: MemberId | number; memberName: string },
    maybeName?: string,
  ): Promise<Member> {
    const id = typeof idOrInput === "object" ? idOrInput.id : idOrInput;
    const rawName = typeof idOrInput === "object" ? idOrInput.memberName : (maybeName ?? "");
    const updated = await this.#transport.updateMember(id, { memberName: rawName });
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          members: Object.freeze(this.#state.data.members.map((m) => (m.id === id ? updated : m))),
        }),
      };
    }
    return updated;
  }

  async deleteMember(id: MemberId | number): Promise<void> {
    await this.#transport.deleteMember(id);
    if (this.#state.status === "success") {
      const newMembers = this.#state.data.members.filter((m) => m.id !== id);
      const newAccounts = this.#state.data.accounts.filter((a) => a.ownerMemberId !== id);
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          members: Object.freeze(newMembers),
          accounts: Object.freeze(newAccounts),
        }),
      };
      if (this.#selectedMemberId === id) {
        this.#selectedMemberId = newMembers[0]?.id ?? null;
      }
    }
  }

  async addBankAccount(
    input: Omit<CreateBankAccountInput, "familyId" | "currencyId"> & {
      familyId?: FamilyId | number;
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    const familyId = input.familyId ?? this.family?.id ?? 1;
    let currencyId = input.currencyId;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.currencies.find((c) => c.code === input.currency)?.id;
    }
    const fullPayload: CreateBankAccountInput = {
      familyId,
      ownerMemberId: input.ownerMemberId,
      currencyId: currencyId ?? this.currencyId,
      bankName: input.bankName,
      accountName: input.accountName,
      last4: input.last4,
      availableBalanceCents: input.availableBalanceCents,
    };
    const newAcc = await this.#transport.createBankAccount(fullPayload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          accounts: Object.freeze([...this.#state.data.accounts, newAcc]),
        }),
      };
    }
    return newAcc;
  }

  async updateBankAccount(
    idOrInput:
      | AccountId
      | number
      | (Omit<UpdateBankAccountInput, "currencyId"> & {
          id: AccountId | number;
          currencyId?: CurrencyId | number;
          currency?: CurrencyCode;
        }),
    maybeInput?: Omit<UpdateBankAccountInput, "currencyId"> & {
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    let id: AccountId | number;
    let inputData: Omit<UpdateBankAccountInput, "currencyId"> & {
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    };

    if (typeof idOrInput === "number") {
      if (maybeInput === undefined) {
        throw new Error("Missing updateBankAccount payload");
      }
      id = idOrInput as AccountId;
      inputData = maybeInput;
    } else {
      id = idOrInput.id;
      inputData = idOrInput;
    }

    let currencyId = inputData.currencyId;
    if (currencyId === undefined && inputData.currency !== undefined) {
      currencyId = this.currencies.find((c) => c.code === inputData.currency)?.id;
    }
    const payload: UpdateBankAccountInput = {
      currencyId: currencyId ?? this.currencyId,
      bankName: inputData.bankName,
      accountName: inputData.accountName,
      last4: inputData.last4,
      availableBalanceCents: inputData.availableBalanceCents,
    };
    const updated = await this.#transport.updateBankAccount(id, payload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          accounts: Object.freeze(
            this.#state.data.accounts.map((a) => (a.id === id ? updated : a)),
          ),
        }),
      };
    }
    return updated;
  }

  async addCreditCard(
    input: Omit<CreateCreditCardInput, "familyId" | "currencyId"> & {
      familyId?: FamilyId | number;
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    const familyId = input.familyId ?? this.family?.id ?? 1;
    let currencyId = input.currencyId;
    if (currencyId === undefined && input.currency !== undefined) {
      currencyId = this.currencies.find((c) => c.code === input.currency)?.id;
    }
    const fullPayload: CreateCreditCardInput = {
      familyId,
      ownerMemberId: input.ownerMemberId,
      currencyId: currencyId ?? this.currencyId,
      bankName: input.bankName,
      cardName: input.cardName,
      last4: input.last4,
      creditLimitCents: input.creditLimitCents,
      availableCents: input.availableCents,
    };
    const newCard = await this.#transport.createCreditCard(fullPayload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          accounts: Object.freeze([...this.#state.data.accounts, newCard]),
        }),
      };
    }
    return newCard;
  }

  async updateCreditCard(
    idOrInput:
      | AccountId
      | number
      | (Omit<UpdateCreditCardInput, "currencyId"> & {
          id: AccountId | number;
          currencyId?: CurrencyId | number;
          currency?: CurrencyCode;
        }),
    maybeInput?: Omit<UpdateCreditCardInput, "currencyId"> & {
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    let id: AccountId | number;
    let inputData: Omit<UpdateCreditCardInput, "currencyId"> & {
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    };

    if (typeof idOrInput === "number") {
      if (maybeInput === undefined) {
        throw new Error("Missing updateCreditCard payload");
      }
      id = idOrInput as AccountId;
      inputData = maybeInput;
    } else {
      id = idOrInput.id;
      inputData = idOrInput;
    }

    let currencyId = inputData.currencyId;
    if (currencyId === undefined && inputData.currency !== undefined) {
      currencyId = this.currencies.find((c) => c.code === inputData.currency)?.id;
    }
    const payload: UpdateCreditCardInput = {
      currencyId: currencyId ?? this.currencyId,
      bankName: inputData.bankName,
      cardName: inputData.cardName,
      last4: inputData.last4,
      creditLimitCents: inputData.creditLimitCents,
      availableCents: inputData.availableCents,
    };
    const updated = await this.#transport.updateCreditCard(id, payload);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          accounts: Object.freeze(
            this.#state.data.accounts.map((a) => (a.id === id ? updated : a)),
          ),
        }),
      };
    }
    return updated;
  }

  async deleteAccount(id: AccountId | number): Promise<void> {
    await this.#transport.deleteAccount(id);
    if (this.#state.status === "success") {
      this.#state = {
        status: "success",
        data: Object.freeze({
          ...this.#state.data,
          accounts: Object.freeze(this.#state.data.accounts.filter((a) => a.id !== id)),
        }),
      };
    }
  }

  reset(): void {
    this.#state = { status: "idle" };
    this.#selectedCurrencyId = 1 as CurrencyId;
    this.#selectedMemberId = null;
  }
}

export const familyStore = new FamilyStore();
