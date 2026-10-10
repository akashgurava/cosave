/**
 * Reactive family and accounts store managing family state, members, and instruments.
 *
 * Encapsulates family metadata, member rosters, and depository/credit accounts behind
 * private Svelte 5 state runes, reactive O(1) relational lookups, and explicit action methods.
 */

import { SvelteMap } from "svelte/reactivity";
import { ApiError } from "$lib/api";
import { expectPresent, type AsyncState, type MinorUnits } from "$lib/types";
import { errorToToast } from "$lib/toast";
import { familyApi } from "./api";
import { formatMoney, getBrowserRegion, getCurrencyScale, getCurrencySymbol } from "./currency";
import type {
  Account,
  AccountId,
  BankAccount,
  CreateBankAccountInput,
  CreateCreditCardInput,
  CreateFamilyInput,
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

  // Reactive derived O(1) indices
  #currencyByIdMap = $derived(new SvelteMap(this.currencies.map((c) => [c.id, c])));
  #currencyByCodeMap = $derived(new SvelteMap(this.currencies.map((c) => [c.code, c])));
  #memberByIdMap = $derived(new SvelteMap(this.members.map((m) => [m.id, m])));
  #accountByIdMap = $derived(new SvelteMap(this.accounts.map((a) => [a.id, a])));

  get currencies(): readonly CurrencyOption[] {
    return this.#state.status === "success" ? this.#state.data.currencies : [];
  }

  get family(): Family | null {
    return this.#state.status === "success" ? this.#state.data.family : null;
  }

  get currencyId(): CurrencyId | number {
    return this.family !== null ? this.family.currencyId : this.#selectedCurrencyId;
  }

  get currency(): CurrencyCode {
    if (this.#state.status === "success") {
      const found = this.#currencyByIdMap.get(this.currencyId as CurrencyId);
      if (found !== undefined) {
        return found.code;
      }
      if (this.currencies[0] !== undefined) {
        return this.currencies[0].code;
      }
    }
    return "USD";
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

  // Invariant-asserting authoritative getters
  requireCurrency(id: CurrencyId | number): CurrencyOption {
    return expectPresent(
      this.#currencyByIdMap.get(id as CurrencyId),
      "STORE.FAMILY.REQUIRE_CURRENCY",
      `Currency ${id} not found in family store`,
    );
  }

  requireMember(id: MemberId | number): Member {
    return expectPresent(
      this.#memberByIdMap.get(id as MemberId),
      "STORE.FAMILY.REQUIRE_MEMBER",
      `Member ${id} not found in family store`,
    );
  }

  requireAccount(id: AccountId | number): Account {
    return expectPresent(
      this.#accountByIdMap.get(id as AccountId),
      "STORE.FAMILY.REQUIRE_ACCOUNT",
      `Account ${id} not found in family store`,
    );
  }

  // Currency helpers backed by backend metadata
  getCurrencyOption(target?: CurrencyId | number | CurrencyCode): CurrencyOption | undefined {
    if (typeof target === "number") {
      return this.#currencyByIdMap.get(target as CurrencyId);
    }
    if (typeof target === "string") {
      return this.#currencyByCodeMap.get(target);
    }
    return this.#currencyByIdMap.get(this.currencyId as CurrencyId);
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

  #loadPromise: Promise<void> | null = null;

  formatMoney(amount: MinorUnits, target?: CurrencyId | number | CurrencyCode): string {
    const opt = this.getCurrencyOption(target);
    const code = opt !== undefined ? opt.code : typeof target === "string" ? target : this.currency;
    return formatMoney(amount, code, opt);
  }

  async load(force = false): Promise<void> {
    if (this.#loadPromise !== null && force === false) {
      return this.#loadPromise;
    }
    this.#loadPromise = this.#performLoad();
    return this.#loadPromise;
  }

  async #performLoad(): Promise<void> {
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
        } else if (details.currencies[0] !== undefined) {
          this.#selectedCurrencyId = details.currencies[0].id;
        }
      }

      if (
        this.#selectedMemberId === null ||
        details.members.some((m) => m.id === this.#selectedMemberId) === false
      ) {
        const firstMember = details.members[0];
        this.#selectedMemberId = firstMember !== undefined ? firstMember.id : null;
      }
      this.#state = { status: "success", data: details };
    } catch (err) {
      const action =
        err instanceof ApiError && err.action !== null && err.action !== undefined
          ? err.action
          : "CONFIG.FAMILY.LOAD.FAILED";
      const message = err instanceof Error ? err.message : "Failed to load family configuration";
      this.#state = { status: "error", error: { action, message } };
    }
  }

  async createFamily(input: CreateFamilyInput): Promise<Family> {
    try {
      const created = await this.#transport.createFamily(input);
      this.#selectedCurrencyId = created.currencyId;
      if (this.#state.status === "success") {
        this.#state = {
          status: "success",
          data: Object.freeze({
            ...this.#state.data,
            family: created,
          }),
        };
      }
      return created;
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async updateFamily(input: UpdateFamilyInput): Promise<Family> {
    try {
      expectPresent(
        this.family,
        "STORE.FAMILY.UPDATE_FAMILY",
        "Cannot update family without an initialized family",
      );
      const updated = await this.#transport.updateFamily({
        familyName: input.familyName,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  getMember(id: MemberId | number | null | undefined): Member | null {
    if (id === null || id === undefined) {
      return null;
    }
    const found = this.#memberByIdMap.get(id as MemberId);
    return found !== undefined ? found : null;
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
    try {
      const rawName = typeof inputOrName === "string" ? inputOrName : inputOrName.memberName;
      const family = expectPresent(
        this.family,
        "STORE.FAMILY.ADD_MEMBER",
        "Cannot add member without an initialized family",
      );
      const newMember = await this.#transport.createMember({
        familyId: family.id,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async updateMember(
    idOrInput: MemberId | number | { id: MemberId | number; memberName: string },
    maybeName?: string,
  ): Promise<Member> {
    try {
      const id = typeof idOrInput === "object" ? idOrInput.id : idOrInput;
      const rawName =
        typeof idOrInput === "object"
          ? idOrInput.memberName
          : maybeName !== null && maybeName !== undefined
            ? maybeName
            : "";
      const updated = await this.#transport.updateMember(id, { memberName: rawName });
      if (this.#state.status === "success") {
        this.#state = {
          status: "success",
          data: Object.freeze({
            ...this.#state.data,
            members: Object.freeze(
              this.#state.data.members.map((m) => (m.id === id ? updated : m)),
            ),
          }),
        };
      }
      return updated;
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async deleteMember(id: MemberId | number): Promise<void> {
    try {
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
          const firstMem = newMembers[0];
          this.#selectedMemberId = firstMem !== undefined ? firstMem.id : null;
        }
      }
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async addBankAccount(
    input: Omit<CreateBankAccountInput, "familyId" | "currencyId"> & {
      familyId?: FamilyId | number;
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<BankAccount> {
    try {
      const family = expectPresent(
        this.family,
        "STORE.FAMILY.ADD_BANK_ACCOUNT",
        "Cannot add bank account without an initialized family",
      );
      const familyId = input.familyId !== undefined ? input.familyId : family.id;
      let currencyId = input.currencyId;
      if (currencyId === undefined && input.currency !== undefined) {
        const foundCurr = this.#currencyByCodeMap.get(input.currency);
        if (foundCurr !== undefined) {
          currencyId = foundCurr.id;
        }
      }
      const fullPayload: CreateBankAccountInput = {
        familyId,
        ownerMemberId: input.ownerMemberId,
        currencyId: currencyId !== undefined ? currencyId : family.currencyId,
        bankName: input.bankName,
        accountName: input.accountName,
        last4: input.last4,
        availableBalance: input.availableBalance,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
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
    try {
      const family = expectPresent(
        this.family,
        "STORE.FAMILY.UPDATE_BANK_ACCOUNT",
        "Cannot update bank account without an initialized family",
      );
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
        const foundCurr = this.#currencyByCodeMap.get(inputData.currency);
        if (foundCurr !== undefined) {
          currencyId = foundCurr.id;
        }
      }
      const payload: UpdateBankAccountInput = {
        currencyId: currencyId !== undefined ? currencyId : family.currencyId,
        bankName: inputData.bankName,
        accountName: inputData.accountName,
        last4: inputData.last4,
        availableBalance: inputData.availableBalance,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async addCreditCard(
    input: Omit<CreateCreditCardInput, "familyId" | "currencyId"> & {
      familyId?: FamilyId | number;
      currencyId?: CurrencyId | number;
      currency?: CurrencyCode;
    },
  ): Promise<CreditCardAccount> {
    try {
      const family = expectPresent(
        this.family,
        "STORE.FAMILY.ADD_CREDIT_CARD",
        "Cannot add credit card without an initialized family",
      );
      const familyId = input.familyId !== undefined ? input.familyId : family.id;
      let currencyId = input.currencyId;
      if (currencyId === undefined && input.currency !== undefined) {
        const foundCurr = this.#currencyByCodeMap.get(input.currency);
        if (foundCurr !== undefined) {
          currencyId = foundCurr.id;
        }
      }
      const fullPayload: CreateCreditCardInput = {
        familyId,
        ownerMemberId: input.ownerMemberId,
        currencyId: currencyId !== undefined ? currencyId : family.currencyId,
        bankName: input.bankName,
        cardName: input.cardName,
        last4: input.last4,
        creditLimit: input.creditLimit,
        availableCredit: input.availableCredit,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
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
    try {
      const family = expectPresent(
        this.family,
        "STORE.FAMILY.UPDATE_CREDIT_CARD",
        "Cannot update credit card without an initialized family",
      );
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
        const foundCurr = this.#currencyByCodeMap.get(inputData.currency);
        if (foundCurr !== undefined) {
          currencyId = foundCurr.id;
        }
      }
      const payload: UpdateCreditCardInput = {
        currencyId: currencyId !== undefined ? currencyId : family.currencyId,
        bankName: inputData.bankName,
        cardName: inputData.cardName,
        last4: inputData.last4,
        creditLimit: inputData.creditLimit,
        availableCredit: inputData.availableCredit,
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async deleteAccount(id: AccountId | number): Promise<void> {
    try {
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
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  reset(): void {
    this.#state = { status: "idle" };
    this.#selectedCurrencyId = 1 as CurrencyId;
    this.#selectedMemberId = null;
    this.#loadPromise = null;
  }
}

export const familyStore = new FamilyStore();
