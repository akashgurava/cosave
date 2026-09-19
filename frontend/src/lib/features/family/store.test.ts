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

  it("calculates aggregate credit limit correctly across all credit card accounts", () => {
    const totalLimit = familyStore.accounts
      .filter((a) => a.type === "credit_card")
      .reduce((sum, a) => sum + (a.type === "credit_card" ? a.creditLimit : 0), 0);

    expect(familyStore.totalCreditLimit).toBe(totalLimit);
  });

  it("adds, updates, and deletes members cleanly", () => {
    const newMember = familyStore.addMember({ name: "Charlie Miller" });
    expect(newMember.name).toBe("Charlie Miller");
    expect(familyStore.getMember(newMember.id)?.name).toBe("Charlie Miller");

    const updated = familyStore.updateMember({ id: newMember.id, name: "Charles Miller" });
    expect(updated).toBe(true);
    expect(familyStore.getMember(newMember.id)?.name).toBe("Charles Miller");

    familyStore.deleteMember(newMember.id);
    expect(familyStore.getMember(newMember.id)).toBeNull();
  });

  it("adds, updates, and deletes bank accounts", () => {
    const firstMember = familyStore.members[0];
    expect(firstMember).toBeDefined();

    const bankAcc = familyStore.addBankAccount({
      ownerMemberId: firstMember.id,
      bankName: "Ally",
      last4: "9912",
    });

    expect(bankAcc.bankName).toBe("Ally");
    expect(bankAcc.last4).toBe("9912");
    expect(familyStore.getMemberBankAccounts(firstMember.id)).toContainEqual(bankAcc);

    const updated = familyStore.updateBankAccount({
      id: bankAcc.id,
      bankName: "Ally Bank",
      last4: "9912",
    });
    expect(updated).toBe(true);

    familyStore.deleteAccount(bankAcc.id);
    expect(familyStore.getMemberBankAccounts(firstMember.id).some((a) => a.id === bankAcc.id)).toBe(
      false,
    );
  });

  it("adds, updates, and deletes credit cards", () => {
    const firstMember = familyStore.members[0];
    expect(firstMember).toBeDefined();

    const card = familyStore.addCreditCard({
      ownerMemberId: firstMember.id,
      bankName: "Amex",
      cardName: "Gold",
      last4: "1001",
      creditLimit: 15000,
    });

    expect(card.cardName).toBe("Gold");
    expect(card.creditLimit).toBe(15000);
    expect(familyStore.getMemberCreditCards(firstMember.id)).toContainEqual(card);

    const updated = familyStore.updateCreditCard({
      id: card.id,
      bankName: "Amex",
      cardName: "Rose Gold",
      last4: "1001",
      creditLimit: 20000,
    });
    expect(updated).toBe(true);

    familyStore.deleteAccount(card.id);
    expect(familyStore.getMemberCreditCards(firstMember.id).some((a) => a.id === card.id)).toBe(
      false,
    );
  });
});
