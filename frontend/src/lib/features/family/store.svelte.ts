/**
 * Reactive family and accounts store managing family state, members, and instruments.
 *
 * Encapsulates family metadata, member rosters, and depository/credit accounts behind
 * private Svelte 5 state runes, reactive O(1) relational lookups, and explicit action methods.
 */

import { SvelteMap } from "svelte/reactivity";
import { ApiError } from "$lib/api";
import { expectPresent, type AsyncState } from "$lib/types";
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
  MinorUnits,
  UpdateBankAccountInput,
  UpdateCreditCardInput,
  UpdateFamilyInput,
  UpdateMemberInput,
} from "./types";

export class FamilyStore {
  #state = $state<AsyncState<FamilyDetails>>({ status: "idle" });
  #selectedCurrencyId = $state<CurrencyId | null>(null);
  #selectedMemberId = $state<MemberId | null>(null);
  #transport: FamilyTransport;
  #loadPromise: Promise<void> | null = null;

  constructor(transport: FamilyTransport = familyApi) {
    this.#transport = transport;
  }

  get state(): AsyncState<FamilyDetails> {
    return this.#state;
  }

  get isLoading(): boolean {
    return this.#state.status === "loading";
  }

  get isSuccess(): boolean {
    return this.#state.status === "success";
  }

  get error(): string | null {
    return this.#state.status === "error" ? this.#state.error.message : null;
  }

  // Authoritative O(1) relational Map lookups
  #currencyByIdMap = $derived(new SvelteMap(this.currencies.map((c) => [c.id, c])));
  #currencyByCodeMap = $derived(new SvelteMap(this.currencies.map((c) => [c.code, c])));
  #memberByIdMap = $derived(new SvelteMap(this.members.map((m) => [m.id, m])));
  #accountByIdMap = $derived(new SvelteMap(this.accounts.map((a) => [a.id, a])));

  get currencies(): readonly CurrencyOption[] {
    return this.#state.status === "success" ? this.#state.data.currencies : [];
  }

  get #family(): Family | null {
    return this.#state.status === "success" ? this.#state.data.family : null;
  }

  /**
   * Authoritative active workspace currency option. Fails fast if family or currency is not loaded.
   */
  requireCurrency(): CurrencyOption {
    const family = this.requireFamily();
    return expectPresent(
      this.#currencyByIdMap.get(family.currencyId),
      "CONFIG.FAMILY.REQUIRE_CURRENCY",
      `Active workspace currency ${family.currencyId} not found in family store`,
    );
  }

  /**
   * Authoritative active currency code. Fails fast if family or currency is not loaded.
   */
  get currency(): CurrencyCode {
    return this.requireCurrency().code;
  }

  get members(): readonly Member[] {
    return this.#state.status === "success" ? this.#state.data.members : [];
  }

  get needsOnboarding(): boolean {
    return this.isSuccess === true && this.#family === null;
  }

  get accounts(): readonly Account[] {
    return this.#state.status === "success" ? this.#state.data.accounts : [];
  }

  get selectedMemberId(): MemberId | null {
    return this.#selectedMemberId;
  }

  set selectedMemberId(id: MemberId | null) {
    this.#selectedMemberId = id;
  }

  // Invariant-asserting authoritative getters
  requireFamily(): Family {
    return expectPresent(
      this.#family,
      "CONFIG.FAMILY.REQUIRE_FAMILY",
      "Family workspace required. Ensure onboarding has been completed before accessing family.",
    );
  }

  requireMember(id: MemberId): Member {
    return expectPresent(
      this.#memberByIdMap.get(id),
      "CONFIG.FAMILY.REQUIRE_MEMBER",
      `Member ${id} not found in family store`,
    );
  }

  requireAccount(id: AccountId): Account {
    return expectPresent(
      this.#accountByIdMap.get(id),
      "CONFIG.FAMILY.REQUIRE_ACCOUNT",
      `Account ${id} not found in family store`,
    );
  }

  getMember(id: MemberId | null | undefined): Member | null {
    if (id === null || id === undefined) {
      return null;
    }
    const found = this.#memberByIdMap.get(id);
    return found !== undefined ? found : null;
  }

  getMemberAccounts(memberId: MemberId): readonly Account[] {
    return this.accounts.filter((a) => a.ownerMemberId === memberId);
  }

  getMemberBankAccounts(memberId: MemberId): readonly BankAccount[] {
    return this.accounts.filter(
      (a): a is BankAccount => a.ownerMemberId === memberId && a.type === "bank_account",
    );
  }

  getMemberCreditCards(memberId: MemberId): readonly CreditCardAccount[] {
    return this.accounts.filter(
      (a): a is CreditCardAccount => a.ownerMemberId === memberId && a.type === "credit_card",
    );
  }

  // Currency helpers backed by backend metadata
  getCurrencyOption(target?: CurrencyId | CurrencyCode): CurrencyOption | undefined {
    if (typeof target === "number") {
      return this.#currencyByIdMap.get(target);
    }
    if (typeof target === "string") {
      return this.#currencyByCodeMap.get(target);
    }
    if (this.#family !== null) {
      return this.#currencyByIdMap.get(this.#family.currencyId);
    }
    if (this.#selectedCurrencyId !== null) {
      return this.#currencyByIdMap.get(this.#selectedCurrencyId);
    }
    return undefined;
  }

  getCurrencySymbol(target?: CurrencyId | CurrencyCode): string {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.symbol;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencySymbol(code);
  }

  getCurrencyScale(target?: CurrencyId | CurrencyCode): number {
    const opt = this.getCurrencyOption(target);
    if (opt !== undefined) {
      return opt.scale;
    }
    const code = typeof target === "string" ? target : this.currency;
    return getCurrencyScale(code);
  }

  formatMoney(amount: MinorUnits, target?: CurrencyId | CurrencyCode): string {
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
        } else {
          const firstCurrency = details.currencies[0];
          this.#selectedCurrencyId = firstCurrency !== undefined ? firstCurrency.id : null;
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
      errorToToast(err);
    }
  }

  async createFamily(input: CreateFamilyInput): Promise<Family> {
    try {
      const created = await this.#transport.createFamily(input);
      this.#selectedCurrencyId = created.currencyId;
      const currentData =
        this.#state.status === "success"
          ? this.#state.data
          : { family: null, members: [], accounts: [], currencies: [] };

      this.#state = {
        status: "success",
        data: Object.freeze({
          ...currentData,
          family: created,
        }),
      };
      return created;
    } catch (err) {
      errorToToast(err);
      throw err;
    }
  }

  async updateFamily(input: UpdateFamilyInput): Promise<Family> {
    try {
      this.requireFamily();
      const updated = await this.#transport.updateFamily(input);
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

  async addMember(inputOrName: string | CreateMemberInput): Promise<Member> {
    try {
      const family = this.requireFamily();
      const payload: CreateMemberInput =
        typeof inputOrName === "string"
          ? { familyId: family.id, memberName: inputOrName }
          : inputOrName;

      const newMember = await this.#transport.createMember(payload);
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

  async updateMember(id: MemberId, inputOrName: string | UpdateMemberInput): Promise<Member> {
    try {
      const payload: UpdateMemberInput =
        typeof inputOrName === "string" ? { memberName: inputOrName } : inputOrName;

      const updated = await this.#transport.updateMember(id, payload);
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

  async deleteMember(id: MemberId): Promise<void> {
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
    input: Omit<CreateBankAccountInput, "familyId"> & { familyId?: FamilyId },
  ): Promise<BankAccount> {
    try {
      const family = this.requireFamily();
      const fullPayload: CreateBankAccountInput = {
        familyId: input.familyId !== undefined ? input.familyId : family.id,
        ownerMemberId: input.ownerMemberId,
        currencyId: input.currencyId !== undefined ? input.currencyId : family.currencyId,
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

  async updateBankAccount(id: AccountId, input: UpdateBankAccountInput): Promise<BankAccount> {
    try {
      const updated = await this.#transport.updateBankAccount(id, input);
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
    input: Omit<CreateCreditCardInput, "familyId"> & { familyId?: FamilyId },
  ): Promise<CreditCardAccount> {
    try {
      const family = this.requireFamily();
      const fullPayload: CreateCreditCardInput = {
        familyId: input.familyId !== undefined ? input.familyId : family.id,
        ownerMemberId: input.ownerMemberId,
        currencyId: input.currencyId !== undefined ? input.currencyId : family.currencyId,
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

  async updateCreditCard(id: AccountId, input: UpdateCreditCardInput): Promise<CreditCardAccount> {
    try {
      const updated = await this.#transport.updateCreditCard(id, input);
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

  async deleteAccount(id: AccountId): Promise<void> {
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
    this.#selectedCurrencyId = null;
    this.#selectedMemberId = null;
    this.#loadPromise = null;
  }
}

export const familyStore = new FamilyStore();
