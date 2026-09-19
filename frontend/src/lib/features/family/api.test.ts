import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  api,
  ApiError,
  Code,
  ContractViolationError,
  MemoryTransportAdapter,
  Status,
} from "$lib/api";
import { familyApi } from "./api";
import type { FamilyOverview, Member, BankAccount, CreditCardAccount } from "./types";

const mockInitialOverview: FamilyOverview = {
  family: {
    id: "fam-1",
    name: "Miller Household",
    createdAt: "2026-01-01T00:00:00Z",
  },
  members: [
    {
      id: "mem-1",
      familyId: "fam-1",
      name: "Sarah Miller",
      createdAt: "2026-01-01T00:00:00Z",
    },
    {
      id: "mem-2",
      familyId: "fam-1",
      name: "David Miller",
      createdAt: "2026-01-02T00:00:00Z",
    },
  ],
  accounts: [
    {
      id: "acc-1",
      familyId: "fam-1",
      ownerMemberId: "mem-1",
      type: "bank_account",
      bankName: "Chase",
      last4: "4821",
      createdAt: "2026-01-01T00:00:00Z",
    },
    {
      id: "acc-2",
      familyId: "fam-1",
      ownerMemberId: "mem-1",
      type: "credit_card",
      bankName: "Chase",
      cardName: "Sapphire Preferred",
      last4: "5561",
      creditLimit: 20000,
      createdAt: "2026-01-02T00:00:00Z",
    },
  ],
};

describe("Family API & Contract Specification (In-Memory Seam & Decoders)", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: () => void;

  beforeEach(() => {
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
  });

  afterEach(() => {
    restoreTransport?.();
  });

  describe("familyApi.getOverview", () => {
    it("fetches and decodes full family overview correctly", async () => {
      memoryTransport.on("GET", "/api/v1/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockInitialOverview,
      }));

      const overview = await familyApi.getOverview();
      expect(overview.family.name).toBe("Miller Household");
      expect(overview.members).toHaveLength(2);
      expect(overview.accounts).toHaveLength(2);

      const bankAcc = overview.accounts[0];
      expect(bankAcc.type).toBe("bank_account");
      expect(bankAcc.bankName).toBe("Chase");

      const creditCard = overview.accounts[1] as CreditCardAccount;
      expect(creditCard.type).toBe("credit_card");
      expect(creditCard.creditLimit).toBe(20000);
    });

    it("throws ContractViolationError when response payload is malformed", async () => {
      memoryTransport.on("GET", "/api/v1/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: { id: "fam-1" }, // missing name and createdAt
          members: "not-an-array",
          accounts: [],
        },
      }));

      await expect(familyApi.getOverview()).rejects.toThrow(ContractViolationError);
    });

    it("throws ContractViolationError when account discriminator is unknown", async () => {
      memoryTransport.on("GET", "/api/v1/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: mockInitialOverview.family,
          members: mockInitialOverview.members,
          accounts: [
            {
              id: "acc-unknown",
              familyId: "fam-1",
              ownerMemberId: "mem-1",
              type: "crypto_wallet",
              bankName: "Ledger",
              last4: "0000",
              createdAt: "2026-01-01T00:00:00Z",
            },
          ],
        },
      }));

      await expect(familyApi.getOverview()).rejects.toThrow(ContractViolationError);
    });
  });

  describe("familyApi.createMember", () => {
    it("creates and decodes a new member", async () => {
      const newMember: Member = {
        id: "mem-3",
        familyId: "fam-1",
        name: "Emma Miller",
        createdAt: "2026-01-03T00:00:00Z",
      };

      memoryTransport.on("POST", "/api/v1/family/members", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.name).toBe("Emma Miller");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: newMember,
        };
      });

      const member = await familyApi.createMember({ name: "Emma Miller" });
      expect(member.id).toBe("mem-3");
      expect(member.name).toBe("Emma Miller");
    });

    it("handles 400 Bad Request error correctly", async () => {
      memoryTransport.on("POST", "/api/v1/family/members", () => {
        throw new ApiError("Member name cannot be empty", 400);
      });

      await expect(familyApi.createMember({ name: "" })).rejects.toThrow(ApiError);
    });
  });

  describe("familyApi.updateMember", () => {
    it("interpolates :id in path and decodes updated member", async () => {
      const updatedMember: Member = {
        id: "mem-1",
        familyId: "fam-1",
        name: "Sarah Miller-Smith",
        createdAt: "2026-01-01T00:00:00Z",
      };

      memoryTransport.on("PUT", "/api/v1/family/members/mem-1", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.name).toBe("Sarah Miller-Smith");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updatedMember,
        };
      });

      const result = await familyApi.updateMember({
        id: "mem-1",
        name: "Sarah Miller-Smith",
      });
      expect(result.name).toBe("Sarah Miller-Smith");
    });
  });

  describe("familyApi.deleteMember", () => {
    it("sends DELETE request with :id path param", async () => {
      let deletedId: string | null = null;
      memoryTransport.on("DELETE", "/api/v1/family/members/mem-2", () => {
        deletedId = "mem-2";
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      await familyApi.deleteMember("mem-2");
      expect(deletedId).toBe("mem-2");
    });
  });

  describe("familyApi.createBankAccount & updateBankAccount", () => {
    it("creates bank account with proper validation", async () => {
      const newBank: BankAccount = {
        id: "acc-3",
        familyId: "fam-1",
        ownerMemberId: "mem-2",
        type: "bank_account",
        bankName: "Ally Bank",
        last4: "9102",
        createdAt: "2026-01-03T00:00:00Z",
      };

      memoryTransport.on("POST", "/api/v1/family/accounts/bank", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newBank,
      }));

      const res = await familyApi.createBankAccount({
        ownerMemberId: "mem-2",
        bankName: "Ally Bank",
        last4: "9102",
      });
      expect(res.id).toBe("acc-3");
      expect(res.type).toBe("bank_account");
      expect(res.bankName).toBe("Ally Bank");
    });

    it("updates bank account details", async () => {
      const updated: BankAccount = {
        id: "acc-1",
        familyId: "fam-1",
        ownerMemberId: "mem-1",
        type: "bank_account",
        bankName: "JPMorgan Chase",
        last4: "4821",
        createdAt: "2026-01-01T00:00:00Z",
      };

      memoryTransport.on("PUT", "/api/v1/family/accounts/bank/acc-1", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updated,
      }));

      const res = await familyApi.updateBankAccount({
        id: "acc-1",
        bankName: "JPMorgan Chase",
        last4: "4821",
      });
      expect(res.bankName).toBe("JPMorgan Chase");
    });
  });

  describe("familyApi.createCreditCard & updateCreditCard", () => {
    it("creates credit card account and parses limit as number", async () => {
      const newCard: CreditCardAccount = {
        id: "acc-4",
        familyId: "fam-1",
        ownerMemberId: "mem-1",
        type: "credit_card",
        bankName: "American Express",
        cardName: "Gold Card",
        last4: "1001",
        creditLimit: 15000,
        createdAt: "2026-01-04T00:00:00Z",
      };

      memoryTransport.on("POST", "/api/v1/family/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newCard,
      }));

      const res = await familyApi.createCreditCard({
        ownerMemberId: "mem-1",
        bankName: "American Express",
        cardName: "Gold Card",
        last4: "1001",
        creditLimit: 15000,
      });
      expect(res.type).toBe("credit_card");
      expect(res.creditLimit).toBe(15000);
    });

    it("throws ContractViolationError if credit limit is missing or non-number", async () => {
      memoryTransport.on("POST", "/api/v1/family/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          id: "acc-invalid",
          familyId: "fam-1",
          ownerMemberId: "mem-1",
          type: "credit_card",
          bankName: "Amex",
          cardName: "Gold",
          last4: "1001",
          creditLimit: "fifteen thousand", // bad type
          createdAt: "2026-01-04T00:00:00Z",
        },
      }));

      await expect(
        familyApi.createCreditCard({
          ownerMemberId: "mem-1",
          bankName: "Amex",
          cardName: "Gold",
          last4: "1001",
          creditLimit: 15000,
        }),
      ).rejects.toThrow(ContractViolationError);
    });

    it("updates credit card details and limit", async () => {
      const updatedCard: CreditCardAccount = {
        id: "acc-2",
        familyId: "fam-1",
        ownerMemberId: "mem-1",
        type: "credit_card",
        bankName: "Chase",
        cardName: "Sapphire Reserve",
        last4: "5561",
        creditLimit: 25000,
        createdAt: "2026-01-02T00:00:00Z",
      };

      memoryTransport.on("PUT", "/api/v1/family/accounts/credit/acc-2", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updatedCard,
      }));

      const res = await familyApi.updateCreditCard({
        id: "acc-2",
        bankName: "Chase",
        cardName: "Sapphire Reserve",
        last4: "5561",
        creditLimit: 25000,
      });
      expect(res.cardName).toBe("Sapphire Reserve");
      expect(res.creditLimit).toBe(25000);
    });
  });

  describe("familyApi.deleteAccount", () => {
    it("deletes account by id", async () => {
      let deletedId: string | null = null;
      memoryTransport.on("DELETE", "/api/v1/family/accounts/acc-1", () => {
        deletedId = "acc-1";
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      await familyApi.deleteAccount("acc-1");
      expect(deletedId).toBe("acc-1");
    });
  });
});
