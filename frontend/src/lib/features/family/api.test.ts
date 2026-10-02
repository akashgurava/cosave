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
import type { BankAccount, CreditCardAccount, Family, FamilyDetails, Member } from "./types";

const mockInitialDetails: FamilyDetails = {
  family: {
    id: 1,
    family_name: "Miller Household",
    currency: "USD",
    created_at: 1704067200,
  },
  members: [
    {
      id: 1,
      family_id: 1,
      member_name: "Sarah Miller",
      created_at: 1704067200,
    },
    {
      id: 2,
      family_id: 1,
      member_name: "David Miller",
      created_at: 1704153600,
    },
  ],
  accounts: [
    {
      id: 101,
      family_id: 1,
      owner_member_id: 1,
      type: "bank_account",
      currency: "USD",
      bank_name: "Chase",
      account_name: "Total Checking",
      last4: "4821",
      available_balance_cents: 845025,
      created_at: 1704067200,
    },
    {
      id: 201,
      family_id: 1,
      owner_member_id: 1,
      type: "credit_card",
      currency: "USD",
      bank_name: "Chase",
      card_name: "Sapphire Preferred",
      last4: "5561",
      credit_limit_cents: 2000000,
      available_cents: 1785000,
      outstanding_cents: 215000,
      created_at: 1704153600,
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

  describe("familyApi.getDetails", () => {
    it("fetches and decodes full family details correctly", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockInitialDetails,
      }));

      const details = await familyApi.getDetails();
      expect(details.family.family_name).toBe("Miller Household");
      expect(details.family.id).toBe(1);
      expect(details.members).toHaveLength(2);
      expect(details.accounts).toHaveLength(2);

      const bankAcc = details.accounts[0];
      expect(bankAcc).toBeDefined();
      if (bankAcc !== undefined) {
        expect(bankAcc.type).toBe("bank_account");
        expect(bankAcc.bank_name).toBe("Chase");
        if (bankAcc.type === "bank_account") {
          expect(bankAcc.account_name).toBe("Total Checking");
          expect(bankAcc.available_balance_cents).toBe(845025);
        }
        expect(bankAcc.id).toBe(101);
      }

      const creditCard = details.accounts[1];
      expect(creditCard).toBeDefined();
      if (creditCard === undefined) return;
      expect(creditCard.type).toBe("credit_card");
      if (creditCard.type === "credit_card") {
        expect(creditCard.credit_limit_cents).toBe(2000000);
        expect(creditCard.available_cents).toBe(1785000);
        expect(creditCard.outstanding_cents).toBe(215000);
        expect(creditCard.id).toBe(201);
      }
    });

    it("throws ContractViolationError when response payload is malformed", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: { id: 1 }, // missing family_name and created_at
          members: "not-an-array",
          accounts: [],
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ContractViolationError);
    });

    it("throws ContractViolationError when account discriminator is unknown", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: mockInitialDetails.family,
          members: mockInitialDetails.members,
          accounts: [
            {
              id: 999,
              family_id: 1,
              owner_member_id: 1,
              type: "crypto_wallet",
              bank_name: "Ledger",
              last4: "0000",
              created_at: 1704067200,
            },
          ],
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ContractViolationError);
    });
  });

  describe("familyApi.updateFamily", () => {
    it("updates family name and currency", async () => {
      const updatedFamily: Family = {
        id: 1,
        family_name: "Miller Clan",
        currency: "EUR",
        created_at: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/family", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.family_name).toBe("Miller Clan");
        expect(parsed.currency).toBe("EUR");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updatedFamily,
        };
      });

      const res = await familyApi.updateFamily({
        family_name: "Miller Clan",
        currency: "EUR",
      });
      expect(res.family_name).toBe("Miller Clan");
      expect(res.currency).toBe("EUR");
    });

    it("throws ContractViolationError when response family is invalid", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { id: "not-a-number" },
      }));

      await expect(familyApi.updateFamily({ family_name: "Test" })).rejects.toThrow(
        ContractViolationError,
      );
    });
  });

  describe("familyApi.getDefaultCurrency", () => {
    it("fetches default currency with optional region query param", async () => {
      memoryTransport.on("GET", "/api/v1/config/currency/default", ({ url }) => {
        expect(url).toContain("region=IN");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: { currency: "INR" },
        };
      });

      const res = await familyApi.getDefaultCurrency("IN");
      expect(res.currency).toBe("INR");
    });
  });

  describe("familyApi.createMember", () => {
    it("creates and decodes a new member", async () => {
      const newMember: Member = {
        id: 3,
        family_id: 1,
        member_name: "Emma Miller",
        created_at: 1704240000,
      };

      memoryTransport.on("POST", "/api/v1/config/members", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.member_name).toBe("Emma Miller");
        expect(parsed.family_id).toBe(1);
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: newMember,
        };
      });

      const member = await familyApi.createMember({ family_id: 1, member_name: "Emma Miller" });
      expect(member.id).toBe(3);
      expect(member.member_name).toBe("Emma Miller");
    });

    it("handles 400 Bad Request error correctly", async () => {
      memoryTransport.on("POST", "/api/v1/config/members", () => {
        throw new ApiError("Member name cannot be empty", 400);
      });

      await expect(familyApi.createMember({ family_id: 1, member_name: "" })).rejects.toThrow(
        ApiError,
      );
    });
  });

  describe("familyApi.updateMember", () => {
    it("interpolates :id in path and decodes updated member", async () => {
      const updatedMember: Member = {
        id: 1,
        family_id: 1,
        member_name: "Sarah Miller-Smith",
        created_at: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/members/1", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.member_name).toBe("Sarah Miller-Smith");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updatedMember,
        };
      });

      const result = await familyApi.updateMember(1, {
        member_name: "Sarah Miller-Smith",
      });
      expect(result.id).toBe(1);
      expect(result.member_name).toBe("Sarah Miller-Smith");
    });
  });

  describe("familyApi.deleteMember", () => {
    it("sends DELETE request with :id path param", async () => {
      let deletedId: number | null = null;
      memoryTransport.on("DELETE", "/api/v1/config/members/2", () => {
        deletedId = 2;
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const res = await familyApi.deleteMember(2);
      expect(deletedId).toBe(2);
      expect(res).toBeNull();
    });
  });

  describe("familyApi.createBankAccount & updateBankAccount", () => {
    it("creates bank account with proper validation", async () => {
      const newBank: BankAccount = {
        id: 103,
        family_id: 1,
        owner_member_id: 2,
        type: "bank_account",
        currency: "USD",
        bank_name: "Ally Bank",
        account_name: "Savings Bucket",
        last4: "9102",
        available_balance_cents: 2500000,
        created_at: 1704240000,
      };

      memoryTransport.on("POST", "/api/v1/config/accounts/bank", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newBank,
      }));

      const res = await familyApi.createBankAccount({
        family_id: 1,
        owner_member_id: 2,
        currency: "USD",
        bank_name: "Ally Bank",
        account_name: "Savings Bucket",
        last4: "9102",
        available_balance_cents: 2500000,
      });
      expect(res.id).toBe(103);
      expect(res.type).toBe("bank_account");
      expect(res.currency).toBe("USD");
      expect(res.bank_name).toBe("Ally Bank");
      expect(res.account_name).toBe("Savings Bucket");
      expect(res.available_balance_cents).toBe(2500000);
    });

    it("updates bank account details", async () => {
      const updated: BankAccount = {
        id: 101,
        family_id: 1,
        owner_member_id: 1,
        type: "bank_account",
        currency: "USD",
        bank_name: "JPMorgan Chase",
        account_name: "Premier Checking",
        last4: "4821",
        available_balance_cents: 950000,
        created_at: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/accounts/bank/101", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updated,
      }));

      const res = await familyApi.updateBankAccount(101, {
        bank_name: "JPMorgan Chase",
        account_name: "Premier Checking",
        last4: "4821",
        available_balance_cents: 950000,
      });
      expect(res.id).toBe(101);
      expect(res.currency).toBe("USD");
      expect(res.bank_name).toBe("JPMorgan Chase");
      expect(res.account_name).toBe("Premier Checking");
      expect(res.available_balance_cents).toBe(950000);
    });
  });

  describe("familyApi.createCreditCard & updateCreditCard", () => {
    it("creates credit card account and parses limits and balances in cents", async () => {
      const newCard: CreditCardAccount = {
        id: 202,
        family_id: 1,
        owner_member_id: 1,
        type: "credit_card",
        currency: "USD",
        bank_name: "American Express",
        card_name: "Gold Card",
        last4: "1001",
        credit_limit_cents: 1500000,
        available_cents: 1300000,
        outstanding_cents: 200000,
        created_at: 1704326400,
      };

      memoryTransport.on("POST", "/api/v1/config/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newCard,
      }));

      const res = await familyApi.createCreditCard({
        family_id: 1,
        owner_member_id: 1,
        currency: "USD",
        bank_name: "American Express",
        card_name: "Gold Card",
        last4: "1001",
        credit_limit_cents: 1500000,
        available_cents: 1300000,
      });
      expect(res.id).toBe(202);
      expect(res.type).toBe("credit_card");
      expect(res.currency).toBe("USD");
      expect(res.credit_limit_cents).toBe(1500000);
      expect(res.available_cents).toBe(1300000);
      expect(res.outstanding_cents).toBe(200000);
    });

    it("throws ContractViolationError if credit limit is missing or non-number", async () => {
      memoryTransport.on("POST", "/api/v1/config/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          id: 299,
          family_id: 1,
          owner_member_id: 1,
          type: "credit_card",
          bank_name: "Amex",
          card_name: "Gold",
          last4: "1001",
          credit_limit_cents: "fifteen thousand", // bad type
          available_cents: 1000000,
          created_at: 1704326400,
        },
      }));

      await expect(
        familyApi.createCreditCard({
          family_id: 1,
          owner_member_id: 1,
          currency: "USD",
          bank_name: "Amex",
          card_name: "Gold",
          last4: "1001",
          credit_limit_cents: 1500000,
          available_cents: 1000000,
        }),
      ).rejects.toThrow(ContractViolationError);
    });

    it("updates credit card details and limits", async () => {
      const updatedCard: CreditCardAccount = {
        id: 201,
        family_id: 1,
        owner_member_id: 1,
        type: "credit_card",
        currency: "USD",
        bank_name: "Chase",
        card_name: "Sapphire Reserve",
        last4: "5561",
        credit_limit_cents: 2500000,
        available_cents: 2100000,
        outstanding_cents: 400000,
        created_at: 1704153600,
      };

      memoryTransport.on("PATCH", "/api/v1/config/accounts/credit/201", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updatedCard,
      }));

      const res = await familyApi.updateCreditCard(201, {
        bank_name: "Chase",
        card_name: "Sapphire Reserve",
        last4: "5561",
        credit_limit_cents: 2500000,
        available_cents: 2100000,
      });
      expect(res.id).toBe(201);
      expect(res.card_name).toBe("Sapphire Reserve");
      expect(res.credit_limit_cents).toBe(2500000);
      expect(res.available_cents).toBe(2100000);
      expect(res.outstanding_cents).toBe(400000);
    });
  });

  describe("familyApi.deleteAccount", () => {
    it("deletes account by id", async () => {
      let deletedId: number | null = null;
      memoryTransport.on("DELETE", "/api/v1/config/accounts/101", () => {
        deletedId = 101;
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: null,
        };
      });

      const res = await familyApi.deleteAccount(101);
      expect(deletedId).toBe(101);
      expect(res).toBeNull();
    });
  });
});
