import { describe, it, expect, beforeEach } from "vitest";
import { familyStore } from "./store.svelte";

describe("familyStore (Presentation Layer Mirror)", () => {
  beforeEach(() => {
    familyStore.resetToDefaults();
  });

  it("initializes with mock family, members, and accounts", () => {
    expect(familyStore.family.name).toBe("The Miller Family");
    expect(familyStore.members.length).toBeGreaterThanOrEqual(2);
    expect(familyStore.accounts.length).toBeGreaterThanOrEqual(2);
  });

  it("calculates aggregate limits, available, and outstanding credit correctly", () => {
    const totalLimitCents = familyStore.accounts
      .filter((a) => a.type === "credit_card")
      .reduce((sum, a) => sum + (a.type === "credit_card" ? a.credit_limit_cents : 0), 0);

    const totalAvailableCents = familyStore.accounts
      .filter((a) => a.type === "credit_card")
      .reduce((sum, a) => sum + (a.type === "credit_card" ? a.available_cents : 0), 0);

    const totalOutstandingCents = familyStore.accounts
      .filter((a) => a.type === "credit_card")
      .reduce((sum, a) => sum + (a.type === "credit_card" ? a.outstanding_cents : 0), 0);

    expect(familyStore.totalCreditLimitCents).toBe(totalLimitCents);
    expect(familyStore.totalAvailableCreditCents).toBe(totalAvailableCents);
    expect(familyStore.totalOutstandingCreditCents).toBe(totalOutstandingCents);
    expect(familyStore.totalCreditLimit).toBe(Math.round(totalLimitCents / 100));
    expect(familyStore.totalAvailableCredit).toBe(Math.round(totalAvailableCents / 100));
    expect(familyStore.totalOutstandingCredit).toBe(Math.round(totalOutstandingCents / 100));
  });

  it("adds, updates, and deletes members cleanly", async () => {
    const newMember = await familyStore.addMember({ name: "Charlie Miller" });
    expect(newMember.name).toBe("Charlie Miller");
    expect(typeof newMember.id).toBe("number");
    expect(familyStore.getMember(newMember.id)?.name).toBe("Charlie Miller");

    const updated = await familyStore.updateMember({ id: newMember.id, name: "Charles Miller" });
    expect(updated.name).toBe("Charles Miller");
    expect(familyStore.getMember(newMember.id)?.name).toBe("Charles Miller");

    await familyStore.deleteMember(newMember.id);
    expect(familyStore.getMember(newMember.id)).toBeNull();
  });

  it("adds, updates, and deletes bank accounts with account_name and available_balance_cents", async () => {
    const firstMember = familyStore.members[0];
    expect(firstMember).toBeDefined();
    if (firstMember === undefined) return;

    const bankAcc = await familyStore.addBankAccount({
      owner_member_id: firstMember.id,
      bank_name: "Ally",
      account_name: "Savings",
      last4: "9912",
      available_balance_cents: 250000,
    });

    expect(typeof bankAcc.id).toBe("number");
    expect(bankAcc.bank_name).toBe("Ally");
    expect(bankAcc.account_name).toBe("Savings");
    expect(bankAcc.last4).toBe("9912");
    expect(bankAcc.available_balance_cents).toBe(250000);
    expect(familyStore.getMemberBankAccounts(firstMember.id)).toContainEqual(bankAcc);

    const updated = await familyStore.updateBankAccount({
      id: bankAcc.id,
      bank_name: "Ally Bank",
      account_name: "High Yield Savings",
      last4: "9912",
      available_balance_cents: 300000,
    });
    expect(updated.bank_name).toBe("Ally Bank");

    const retrieved = familyStore
      .getMemberBankAccounts(firstMember.id)
      .find((a) => a.id === bankAcc.id);
    expect(retrieved?.bank_name).toBe("Ally Bank");
    expect(retrieved?.account_name).toBe("High Yield Savings");
    expect(retrieved?.available_balance_cents).toBe(300000);

    await familyStore.deleteAccount(bankAcc.id);
    expect(familyStore.getMemberBankAccounts(firstMember.id).some((a) => a.id === bankAcc.id)).toBe(
      false,
    );
  });

  it("adds, updates, and deletes credit cards calculating outstanding as limit - available", async () => {
    const firstMember = familyStore.members[0];
    expect(firstMember).toBeDefined();
    if (firstMember === undefined) return;

    const card = await familyStore.addCreditCard({
      owner_member_id: firstMember.id,
      bank_name: "Amex",
      card_name: "Gold",
      last4: "1001",
      credit_limit_cents: 1500000,
      available_cents: 1200000,
    });

    expect(typeof card.id).toBe("number");
    expect(card.card_name).toBe("Gold");
    expect(card.credit_limit_cents).toBe(1500000);
    expect(card.available_cents).toBe(1200000);
    expect(card.outstanding_cents).toBe(300000); // 1,500,000 - 1,200,000
    expect(familyStore.getMemberCreditCards(firstMember.id)).toContainEqual(card);

    const updated = await familyStore.updateCreditCard({
      id: card.id,
      bank_name: "Amex",
      card_name: "Rose Gold",
      last4: "1001",
      credit_limit_cents: 2000000,
      available_cents: 1700000,
    });
    expect(updated.card_name).toBe("Rose Gold");

    const retrieved = familyStore
      .getMemberCreditCards(firstMember.id)
      .find((c) => c.id === card.id);
    expect(retrieved?.card_name).toBe("Rose Gold");
    expect(retrieved?.credit_limit_cents).toBe(2000000);
    expect(retrieved?.available_cents).toBe(1700000);
    expect(retrieved?.outstanding_cents).toBe(300000); // 2,000,000 - 1,700,000

    await familyStore.deleteAccount(card.id);
    expect(familyStore.getMemberCreditCards(firstMember.id).some((a) => a.id === card.id)).toBe(
      false,
    );
  });

  it("updates family currency and supports account-level currencies", async () => {
    expect(familyStore.currencies.length).toBeGreaterThan(0);
    expect(familyStore.currencies.some((c) => c.code === "INR")).toBe(true);

    familyStore.currency = "EUR";
    expect(familyStore.family.currency).toBe("EUR");
    expect(familyStore.currency).toBe("EUR");

    const firstMember = familyStore.members[0];
    expect(firstMember).toBeDefined();
    if (firstMember === undefined) return;

    const jpyAccount = await familyStore.addBankAccount({
      owner_member_id: firstMember.id,
      currency: "JPY",
      bank_name: "Mizuho",
      account_name: "Tokyo Account",
      last4: "1234",
      available_balance_cents: 50000,
    });
    expect(jpyAccount.currency).toBe("JPY");

    await familyStore.updateBankAccount({
      id: jpyAccount.id,
      currency: "GBP",
      bank_name: "Mizuho UK",
      account_name: "London Account",
      last4: "1234",
      available_balance_cents: 60000,
    });
    const retrieved = familyStore
      .getMemberBankAccounts(firstMember.id)
      .find((a) => a.id === jpyAccount.id);
    expect(retrieved?.currency).toBe("GBP");
  });
});
