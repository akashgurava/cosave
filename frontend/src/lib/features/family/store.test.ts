import { describe, it, expect, vi, beforeEach } from "vitest";
import { FamilyStore, familyStore } from "./store.svelte";
import { familyApi } from "./api";
import type {
  AccountId,
  BankAccount,
  CreditCardAccount,
  CurrencyId,
  CurrencyOption,
  Family,
  FamilyDetails,
  FamilyId,
  Member,
  MemberId,
  MinorUnits,
} from "./types";

const testCurrencies: readonly CurrencyOption[] = [
  { id: 1 as CurrencyId, code: "USD", name: "US Dollar", symbol: "$", scale: 2, sortOrder: 1 },
  { id: 2 as CurrencyId, code: "EUR", name: "Euro", symbol: "€", scale: 2, sortOrder: 2 },
  { id: 3 as CurrencyId, code: "INR", name: "Indian Rupee", symbol: "₹", scale: 2, sortOrder: 3 },
  { id: 4 as CurrencyId, code: "JPY", name: "Japanese Yen", symbol: "¥", scale: 0, sortOrder: 4 },
];

const testFamily: Family = {
  id: 1 as FamilyId,
  familyName: "The Miller Family",
  currencyId: 1 as CurrencyId,
  createdAt: 1704067200,
};

const testMembers: Member[] = [
  { id: 1 as MemberId, familyId: 1 as FamilyId, memberName: "Sarah Miller", createdAt: 1704067200 },
  { id: 2 as MemberId, familyId: 1 as FamilyId, memberName: "David Miller", createdAt: 1704153600 },
];

const testBank: BankAccount = {
  id: 101 as AccountId,
  familyId: 1 as FamilyId,
  ownerMemberId: 1 as MemberId,
  type: "bank_account",
  currencyId: 1 as CurrencyId,
  name: "Total Checking",
  availableBalance: 845025 as MinorUnits,
  createdAt: 1704067200,
};

const testCard: CreditCardAccount = {
  id: 201 as AccountId,
  familyId: 1 as FamilyId,
  ownerMemberId: 1 as MemberId,
  type: "credit_card",
  currencyId: 1 as CurrencyId,
  name: "Sapphire Preferred",
  creditLimit: 2000000 as MinorUnits,
  availableCredit: 1785000 as MinorUnits,
  outstandingBalance: 215000 as MinorUnits,
  createdAt: 1704153600,
};

const testDetails: FamilyDetails = {
  family: testFamily,
  members: testMembers,
  accounts: [testBank, testCard],
  currencies: testCurrencies,
};

describe("FamilyStore (Presentation Layer Mirror of Rust SSOT)", () => {
  let store: FamilyStore;

  beforeEach(() => {
    vi.clearAllMocks();
    store = new FamilyStore();
    familyStore.reset();
  });

  it("initializes empty before load without assuming preloaded state", () => {
    expect(store.isSuccess).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.members).toHaveLength(0);
    expect(store.accounts).toHaveLength(0);
    expect(store.currencies).toHaveLength(0);
    expect(store.selectedMemberId).toBeNull();
    expect(store.needsOnboarding).toBe(false);
  });

  it("computes needsOnboarding correctly when family is null after load", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue({
      ...testDetails,
      family: null,
      members: [],
    });
    vi.spyOn(familyApi, "getDefaultCurrency").mockResolvedValue({ currency: "USD" });

    await store.load();
    expect(store.isSuccess).toBe(true);
    expect(store.needsOnboarding).toBe(true);
  });

  it("loads family, members, accounts, and currencies from familyApi.getDetails", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);

    await store.load();

    expect(store.isSuccess).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.requireFamily().familyName).toBe("The Miller Family");
    expect(store.members).toHaveLength(2);
    expect(store.accounts).toHaveLength(2);
    expect(store.currencies).toHaveLength(4);
    expect(store.selectedMemberId).toBe(1);
  });

  it("delegates addMember, updateMember, and deleteMember to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const newMember: Member = {
      id: 3 as MemberId,
      familyId: 1 as FamilyId,
      memberName: "Charlie Miller",
      createdAt: 1704240000,
    };
    const createSpy = vi.spyOn(familyApi, "createMember").mockResolvedValue(newMember);

    const added = await store.addMember("Charlie Miller");
    expect(createSpy).toHaveBeenCalledWith({ familyId: 1, memberName: "Charlie Miller" });
    expect(added.memberName).toBe("Charlie Miller");
    expect(store.members).toHaveLength(3);

    const updatedMember: Member = { ...newMember, memberName: "Charles Miller" };
    const updateSpy = vi.spyOn(familyApi, "updateMember").mockResolvedValue(updatedMember);

    const updated = await store.updateMember(3 as MemberId, "Charles Miller");
    expect(updateSpy).toHaveBeenCalledWith(3 as MemberId, { memberName: "Charles Miller" });
    expect(updated.memberName).toBe("Charles Miller");
    const foundMem = store.getMember(3 as MemberId);
    expect(foundMem).not.toBeNull();
    if (foundMem !== null) {
      expect(foundMem.memberName).toBe("Charles Miller");
    }

    const deleteSpy = vi.spyOn(familyApi, "deleteMember").mockResolvedValue(null);
    await store.deleteMember(3 as MemberId);
    expect(deleteSpy).toHaveBeenCalledWith(3 as MemberId);
    expect(store.getMember(3 as MemberId)).toBeNull();
  });

  it("delegates addBankAccount, updateBankAccount, and deleteAccount to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const createdBank: BankAccount = {
      id: 102 as AccountId,
      familyId: 1 as FamilyId,
      ownerMemberId: 1 as MemberId,
      type: "bank_account",
      currencyId: 1 as CurrencyId,
      name: "Savings Bucket",
      availableBalance: 250000 as MinorUnits,
      createdAt: 1704240000,
    };
    vi.spyOn(familyApi, "createBankAccount").mockResolvedValue(createdBank);

    const added = await store.addBankAccount({
      ownerMemberId: 1 as MemberId,
      name: "Savings Bucket",
      availableBalance: 250000 as MinorUnits,
    });
    expect(added.id).toBe(102);
    expect(store.getMemberBankAccounts(1 as MemberId)).toHaveLength(2);

    const updatedBank: BankAccount = { ...createdBank, name: "Ally Bank" };
    vi.spyOn(familyApi, "updateBankAccount").mockResolvedValue(updatedBank);

    const updated = await store.updateBankAccount(102 as AccountId, {
      currencyId: 1 as CurrencyId,
      name: "Ally Bank",
      availableBalance: 250000 as MinorUnits,
    });
    expect(updated.name).toBe("Ally Bank");

    vi.spyOn(familyApi, "deleteAccount").mockResolvedValue(null);
    await store.deleteAccount(102 as AccountId);
    expect(store.getMemberBankAccounts(1 as MemberId).some((b) => b.id === 102)).toBe(false);
  });

  it("delegates addCreditCard and updateCreditCard to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const createdCard: CreditCardAccount = {
      id: 202 as AccountId,
      familyId: 1 as FamilyId,
      ownerMemberId: 1 as MemberId,
      type: "credit_card",
      currencyId: 1 as CurrencyId,
      name: "Gold",
      creditLimit: 1500000 as MinorUnits,
      availableCredit: 1200000 as MinorUnits,
      outstandingBalance: 300000 as MinorUnits,
      createdAt: 1704240000,
    };
    vi.spyOn(familyApi, "createCreditCard").mockResolvedValue(createdCard);

    const added = await store.addCreditCard({
      ownerMemberId: 1 as MemberId,
      name: "Gold",
      creditLimit: 1500000 as MinorUnits,
      availableCredit: 1200000 as MinorUnits,
    });
    expect(added.id).toBe(202);
    expect(store.getMemberCreditCards(1 as MemberId)).toHaveLength(2);

    const updatedCard: CreditCardAccount = { ...createdCard, name: "Rose Gold" };
    vi.spyOn(familyApi, "updateCreditCard").mockResolvedValue(updatedCard);

    const updated = await store.updateCreditCard(202 as AccountId, {
      currencyId: 1 as CurrencyId,
      name: "Rose Gold",
      creditLimit: 1500000 as MinorUnits,
      availableCredit: 1200000 as MinorUnits,
    });
    expect(updated.name).toBe("Rose Gold");
  });

  it("resolves currency symbols and scales from backend currencies", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    expect(store.getCurrencySymbol("USD")).toBe("$");
    expect(store.getCurrencySymbol("EUR")).toBe("€");
    expect(store.getCurrencySymbol("INR")).toBe("₹");
    expect(store.getCurrencySymbol("JPY")).toBe("¥");

    expect(store.getCurrencyScale("USD")).toBe(2);
    expect(store.getCurrencyScale("JPY")).toBe(0);

    const formatted = store.formatMoney(50000 as MinorUnits, "JPY");
    expect(formatted).toContain("50,000");
  });

  it("creates family via createFamily and persists to backend", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue({
      ...testDetails,
      family: null,
    });
    vi.spyOn(familyApi, "getDefaultCurrency").mockResolvedValue({ currency: "USD" });
    await store.load();

    const createSpy = vi.spyOn(familyApi, "createFamily").mockResolvedValue({
      ...testFamily,
      currencyId: 2 as CurrencyId,
    });

    await store.createFamily({ familyName: "The Miller Family", currencyId: 2 as CurrencyId });
    expect(store.currency).toBe("EUR");
    expect(createSpy).toHaveBeenCalledWith({
      familyName: "The Miller Family",
      currencyId: 2 as CurrencyId,
    });
  });

  it("supports custom FamilyTransport dependency injection", async () => {
    const mockTransport = {
      getDetails: vi.fn().mockResolvedValue(testDetails),
      getOverview: vi.fn().mockResolvedValue(testDetails),
      getCurrencies: vi.fn().mockResolvedValue(testCurrencies),
      createFamily: vi.fn(),
      updateFamily: vi.fn(),
      getDefaultCurrency: vi.fn().mockResolvedValue({ currency: "USD" }),
      createMember: vi.fn(),
      updateMember: vi.fn(),
      deleteMember: vi.fn(),
      createBankAccount: vi.fn(),
      updateBankAccount: vi.fn(),
      createCreditCard: vi.fn(),
      updateCreditCard: vi.fn(),
      deleteAccount: vi.fn(),
    };

    const injectedStore = new FamilyStore(mockTransport);
    await injectedStore.load();

    expect(mockTransport.getDetails).toHaveBeenCalledTimes(1);
    expect(injectedStore.isSuccess).toBe(true);
    expect(injectedStore.requireFamily().familyName).toBe("The Miller Family");
  });

  it("deduplicates concurrent load calls", async () => {
    const getDetailsSpy = vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);

    const promise1 = store.load();
    const promise2 = store.load();

    await Promise.all([promise1, promise2]);

    expect(getDetailsSpy).toHaveBeenCalledTimes(1);
  });

  it("throws InvariantViolationError when adding member without initialized family", async () => {
    await expect(store.addMember("Charlie")).rejects.toThrow(
      "Family workspace required. Ensure onboarding has been completed before accessing family.",
    );
  });

  it("throws InvariantViolationError when adding account without initialized family or explicit familyId", async () => {
    await expect(
      store.addBankAccount({
        ownerMemberId: 1 as MemberId,
        name: "Checking",
        availableBalance: 1000 as MinorUnits,
      }),
    ).rejects.toThrow(
      "Family workspace required. Ensure onboarding has been completed before accessing family.",
    );

    await expect(
      store.addCreditCard({
        ownerMemberId: 1 as MemberId,
        name: "Gold",
        creditLimit: 10000 as MinorUnits,
        availableCredit: 5000 as MinorUnits,
      }),
    ).rejects.toThrow(
      "Family workspace required. Ensure onboarding has been completed before accessing family.",
    );
  });

  it("resets state and load promise on reset()", async () => {
    const getDetailsSpy = vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();
    expect(store.isSuccess).toBe(true);

    store.reset();
    expect(store.state.status).toBe("idle");
    expect(store.isSuccess).toBe(false);

    await store.load();
    expect(getDetailsSpy).toHaveBeenCalledTimes(2);
  });

  it("authoritatively retrieves member and account or throws InvariantViolationError", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    expect(store.requireMember(1 as MemberId).memberName).toBe("Sarah Miller");
    expect(() => store.requireMember(999 as MemberId)).toThrow(
      "Member 999 not found in family store",
    );

    expect(store.requireAccount(101 as AccountId).name).toBe("Total Checking");
    expect(() => store.requireAccount(999 as AccountId)).toThrow(
      "Account 999 not found in family store",
    );

    expect(store.requireCurrency().code).toBe("USD");
  });

  it("handles load errors and exposes typed error payload", async () => {
    vi.spyOn(familyApi, "getDetails").mockRejectedValue(new Error("Network failed"));

    await store.load();

    expect(store.isSuccess).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.error).toBe("Network failed");
    expect(store.state.status).toBe("error");
  });

  it("updates family name and persists via updateFamily", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const updateSpy = vi.spyOn(familyApi, "updateFamily").mockResolvedValue({
      ...testFamily,
      familyName: "The New Miller Family",
    });

    const updated = await store.updateFamily({
      familyId: 1 as FamilyId,
      familyName: "The New Miller Family",
    });

    expect(updateSpy).toHaveBeenCalledWith({
      familyId: 1 as FamilyId,
      familyName: "The New Miller Family",
    });
    expect(updated.familyName).toBe("The New Miller Family");
    expect(store.requireFamily().familyName).toBe("The New Miller Family");
  });
});
