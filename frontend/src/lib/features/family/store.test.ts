import { describe, it, expect, vi, beforeEach } from "vitest";
import { FamilyStore, familyStore } from "./store.svelte";
import { familyApi } from "./api";
import type {
  BankAccount,
  CreditCardAccount,
  CurrencyOption,
  Family,
  FamilyDetails,
  Member,
} from "./types";

const testCurrencies: readonly CurrencyOption[] = [
  { id: 1, code: "USD", name: "US Dollar", symbol: "$", scale: 2, sortOrder: 1 },
  { id: 2, code: "EUR", name: "Euro", symbol: "€", scale: 2, sortOrder: 2 },
  { id: 3, code: "INR", name: "Indian Rupee", symbol: "₹", scale: 2, sortOrder: 3 },
  { id: 4, code: "JPY", name: "Japanese Yen", symbol: "¥", scale: 0, sortOrder: 4 },
];

const testFamily: Family = {
  id: 1,
  familyName: "The Miller Family",
  currencyId: 1,
  createdAt: 1704067200,
};

const testMembers: Member[] = [
  { id: 1, familyId: 1, memberName: "Sarah Miller", createdAt: 1704067200 },
  { id: 2, familyId: 1, memberName: "David Miller", createdAt: 1704153600 },
];

const testBank: BankAccount = {
  id: 101,
  familyId: 1,
  ownerMemberId: 1,
  type: "bank_account",
  currencyId: 1,
  bankName: "Chase",
  accountName: "Total Checking",
  last4: "4821",
  availableBalanceCents: 845025,
  createdAt: 1704067200,
};

const testCard: CreditCardAccount = {
  id: 201,
  familyId: 1,
  ownerMemberId: 1,
  type: "credit_card",
  currencyId: 1,
  bankName: "Chase",
  cardName: "Sapphire Preferred",
  last4: "5561",
  creditLimitCents: 2000000,
  availableCents: 1785000,
  outstandingCents: 215000,
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
    expect(store.isLoaded).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.members).toHaveLength(0);
    expect(store.accounts).toHaveLength(0);
    expect(store.currencies).toHaveLength(0);
    expect(store.selectedMemberId).toBeNull();
  });

  it("loads family, members, accounts, and currencies from familyApi.getDetails", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);

    await store.load();

    expect(store.isLoaded).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.family?.familyName).toBe("The Miller Family");
    expect(store.members).toHaveLength(2);
    expect(store.accounts).toHaveLength(2);
    expect(store.currencies).toHaveLength(4);
    expect(store.selectedMemberId).toBe(1);
  });

  it("delegates addMember, updateMember, and deleteMember to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const newMember: Member = {
      id: 3,
      familyId: 1,
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

    const updated = await store.updateMember(3, "Charles Miller");
    expect(updateSpy).toHaveBeenCalledWith(3, { memberName: "Charles Miller" });
    expect(updated.memberName).toBe("Charles Miller");
    expect(store.getMember(3)?.memberName).toBe("Charles Miller");

    const deleteSpy = vi.spyOn(familyApi, "deleteMember").mockResolvedValue(null);
    await store.deleteMember(3);
    expect(deleteSpy).toHaveBeenCalledWith(3);
    expect(store.getMember(3)).toBeNull();
  });

  it("delegates addBankAccount, updateBankAccount, and deleteAccount to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const createdBank: BankAccount = {
      id: 102,
      familyId: 1,
      ownerMemberId: 1,
      type: "bank_account",
      currencyId: 1,
      bankName: "Ally",
      accountName: "Savings Bucket",
      last4: "9102",
      availableBalanceCents: 250000,
      createdAt: 1704240000,
    };
    vi.spyOn(familyApi, "createBankAccount").mockResolvedValue(createdBank);

    const added = await store.addBankAccount({
      ownerMemberId: 1,
      bankName: "Ally",
      accountName: "Savings Bucket",
      last4: "9102",
      availableBalanceCents: 250000,
    });
    expect(added.id).toBe(102);
    expect(store.getMemberBankAccounts(1)).toHaveLength(2);

    const updatedBank: BankAccount = { ...createdBank, bankName: "Ally Bank" };
    vi.spyOn(familyApi, "updateBankAccount").mockResolvedValue(updatedBank);

    const updated = await store.updateBankAccount(102, {
      bankName: "Ally Bank",
      accountName: "Savings Bucket",
      last4: "9102",
      availableBalanceCents: 250000,
    });
    expect(updated.bankName).toBe("Ally Bank");

    vi.spyOn(familyApi, "deleteAccount").mockResolvedValue(null);
    await store.deleteAccount(102);
    expect(store.getMemberBankAccounts(1).some((b) => b.id === 102)).toBe(false);
  });

  it("delegates addCreditCard and updateCreditCard to familyApi", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const createdCard: CreditCardAccount = {
      id: 202,
      familyId: 1,
      ownerMemberId: 1,
      type: "credit_card",
      currencyId: 1,
      bankName: "Amex",
      cardName: "Gold",
      last4: "1001",
      creditLimitCents: 1500000,
      availableCents: 1200000,
      outstandingCents: 300000,
      createdAt: 1704240000,
    };
    vi.spyOn(familyApi, "createCreditCard").mockResolvedValue(createdCard);

    const added = await store.addCreditCard({
      ownerMemberId: 1,
      bankName: "Amex",
      cardName: "Gold",
      last4: "1001",
      creditLimitCents: 1500000,
      availableCents: 1200000,
    });
    expect(added.id).toBe(202);
    expect(store.getMemberCreditCards(1)).toHaveLength(2);

    const updatedCard: CreditCardAccount = { ...createdCard, cardName: "Rose Gold" };
    vi.spyOn(familyApi, "updateCreditCard").mockResolvedValue(updatedCard);

    const updated = await store.updateCreditCard(202, {
      bankName: "Amex",
      cardName: "Rose Gold",
      last4: "1001",
      creditLimitCents: 1500000,
      availableCents: 1200000,
    });
    expect(updated.cardName).toBe("Rose Gold");
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

    const formatted = store.formatMoney(50000, "JPY");
    expect(formatted).toContain("50,000");
  });

  it("updates family base currency via setter and persists to backend", async () => {
    vi.spyOn(familyApi, "getDetails").mockResolvedValue(testDetails);
    await store.load();

    const updateSpy = vi.spyOn(familyApi, "updateFamily").mockResolvedValue({
      ...testFamily,
      currencyId: 2,
    });

    store.currency = "EUR";
    expect(store.currency).toBe("EUR");
    expect(updateSpy).toHaveBeenCalledWith({ familyName: "The Miller Family", currencyId: 2 });
  });
});
