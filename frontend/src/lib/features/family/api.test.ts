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
import type {
  BankAccount,
  CreditCardAccount,
  CurrencyOption,
  Family,
  FamilyDetails,
  Member,
} from "./types";

const mockCurrencies: readonly CurrencyOption[] = [
  { id: 1, code: "USD", name: "US Dollar", symbol: "$", scale: 2, sortOrder: 1 },
  { id: 2, code: "EUR", name: "Euro", symbol: "€", scale: 2, sortOrder: 2 },
  { id: 3, code: "INR", name: "Indian Rupee", symbol: "₹", scale: 2, sortOrder: 3 },
  { id: 4, code: "JPY", name: "Japanese Yen", symbol: "¥", scale: 0, sortOrder: 4 },
];

const mockInitialDetails: FamilyDetails = {
  family: {
    id: 1,
    familyName: "Miller Household",
    currencyId: 1,
    createdAt: 1704067200,
  },
  members: [
    {
      id: 1,
      familyId: 1,
      memberName: "Sarah Miller",
      createdAt: 1704067200,
    },
    {
      id: 2,
      familyId: 1,
      memberName: "David Miller",
      createdAt: 1704153600,
    },
  ],
  accounts: [
    {
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
    },
    {
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
    },
  ],
  currencies: mockCurrencies,
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
      expect(details.family?.familyName).toBe("Miller Household");
      expect(details.family?.id).toBe(1);
      expect(details.members).toHaveLength(2);
      expect(details.accounts).toHaveLength(2);

      const bankAcc = details.accounts[0];
      expect(bankAcc).toBeDefined();
      if (bankAcc !== undefined) {
        expect(bankAcc.type).toBe("bank_account");
        expect(bankAcc.bankName).toBe("Chase");
        if (bankAcc.type === "bank_account") {
          expect(bankAcc.accountName).toBe("Total Checking");
          expect(bankAcc.availableBalanceCents).toBe(845025);
        }
        expect(bankAcc.id).toBe(101);
      }

      const creditCard = details.accounts[1];
      expect(creditCard).toBeDefined();
      if (creditCard === undefined) return;
      expect(creditCard.type).toBe("credit_card");
      expect(creditCard.bankName).toBe("Chase");
      if (creditCard.type === "credit_card") {
        expect(creditCard.cardName).toBe("Sapphire Preferred");
        expect(creditCard.creditLimitCents).toBe(2000000);
        expect(creditCard.availableCents).toBe(1785000);
        expect(creditCard.outstandingCents).toBe(215000);
      }
      expect(creditCard.id).toBe(201);
    });

    it("throws ContractViolationError on malformed backend envelope", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: "not-an-object",
          members: [],
          accounts: [],
          currencies: [],
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ContractViolationError);
    });

    it("throws ContractViolationError when an account type is invalid", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          family: mockInitialDetails.family,
          members: mockInitialDetails.members,
          accounts: [
            {
              id: 999,
              familyId: 1,
              ownerMemberId: 1,
              type: "crypto_wallet", // Unsupported
              bankName: "Ledger",
              accountName: "Cold Storage",
              last4: "0000",
              availableBalanceCents: 1000,
              createdAt: 1704067200,
            },
          ],
          currencies: mockCurrencies,
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ContractViolationError);
    });

    it("maps backend ApiError on 401 unauthorized session failure", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Unauthorized,
        status: Status.Unauthenticated,
        data: {
          action: "AUTH.SESSION.MISSING",
          message: "Active user session cookie required",
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ApiError);
      try {
        await familyApi.getDetails();
      } catch (err) {
        expect(err).toBeInstanceOf(ApiError);
        if (err instanceof ApiError) {
          expect(err.code).toBe(Code.Unauthorized);
          expect(err.action).toBe("AUTH.SESSION.MISSING");
          expect(err.message).toContain("Active user session cookie required");
        }
      }
    });

    it("maps backend ApiError on 403 forbidden role failure", async () => {
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: Code.Unauthorized,
        status: Status.Unauthenticated,
        data: {
          action: "AUTH.RBAC.INSUFFICIENT_ROLE",
          message: "Admin role required",
        },
      }));

      await expect(familyApi.getDetails()).rejects.toThrow(ApiError);
      try {
        await familyApi.getDetails();
      } catch (err) {
        expect(err).toBeInstanceOf(ApiError);
        if (err instanceof ApiError) {
          expect(err.code).toBe(Code.Unauthorized);
          expect(err.action).toBe("AUTH.RBAC.INSUFFICIENT_ROLE");
        }
      }
    });
  });

  describe("familyApi.getCurrencies", () => {
    it("fetches and decodes supported currencies list", async () => {
      memoryTransport.on("GET", "/api/v1/config/currencies", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: mockCurrencies,
      }));

      const res = await familyApi.getCurrencies();
      expect(res).toHaveLength(4);
      expect(res[0]?.code).toBe("USD");
      expect(res[0]?.symbol).toBe("$");
      expect(res[0]?.scale).toBe(2);
      expect(res[3]?.code).toBe("JPY");
      expect(res[3]?.scale).toBe(0);
    });

    it("throws ContractViolationError when a currency item is invalid", async () => {
      memoryTransport.on("GET", "/api/v1/config/currencies", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: [{ code: "USD", name: "US Dollar" }], // missing id, symbol and scale
      }));

      await expect(familyApi.getCurrencies()).rejects.toThrow(ContractViolationError);
    });
  });

  describe("familyApi.updateFamily", () => {
    it("updates family name and currency", async () => {
      const updatedFamily: Family = {
        id: 1,
        familyName: "Miller Clan",
        currencyId: 2,
        createdAt: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/family", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.familyName).toBe("Miller Clan");
        expect(parsed.currencyId).toBe(2);
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updatedFamily,
        };
      });

      const res = await familyApi.updateFamily({
        familyName: "Miller Clan",
        currencyId: 2,
      });
      expect(res.familyName).toBe("Miller Clan");
      expect(res.currencyId).toBe(2);
    });

    it("throws ContractViolationError when response family is invalid", async () => {
      memoryTransport.on("PATCH", "/api/v1/config/family", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: { id: "not-a-number" },
      }));

      await expect(
        familyApi.updateFamily({
          familyName: "Broken",
          currencyId: 1,
        }),
      ).rejects.toThrow(ContractViolationError);
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

  describe("familyApi.createMember, updateMember & deleteMember", () => {
    it("creates member and returns valid Member", async () => {
      const newMember: Member = {
        id: 3,
        familyId: 1,
        memberName: "Lucas Miller",
        createdAt: 1704240000,
      };

      memoryTransport.on("POST", "/api/v1/config/members", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.familyId).toBe(1);
        expect(parsed.memberName).toBe("Lucas Miller");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: newMember,
        };
      });

      const res = await familyApi.createMember({
        familyId: 1,
        memberName: "Lucas Miller",
      });
      expect(res.id).toBe(3);
      expect(res.memberName).toBe("Lucas Miller");
    });

    it("maps 409 conflict when duplicate member name is added", async () => {
      memoryTransport.on("POST", "/api/v1/config/members", () => ({
        code: Code.Conflict,
        status: Status.BadRequest,
        data: {
          action: "FAMILY.CREATE_MEMBER.NAME_EXISTS",
          message: "A member named 'Sarah Miller' already exists in this household",
        },
      }));

      await expect(
        familyApi.createMember({
          familyId: 1,
          memberName: "Sarah Miller",
        }),
      ).rejects.toThrow(ApiError);
    });

    it("updates existing member name", async () => {
      const updated: Member = {
        id: 1,
        familyId: 1,
        memberName: "Sarah J. Miller",
        createdAt: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/members/1", ({ body }) => {
        const parsed = JSON.parse(body ?? "{}");
        expect(parsed.memberName).toBe("Sarah J. Miller");
        return {
          code: Code.Zero,
          status: Status.Ok,
          data: updated,
        };
      });

      const res = await familyApi.updateMember(1, { memberName: "Sarah J. Miller" });
      expect(res.id).toBe(1);
      expect(res.memberName).toBe("Sarah J. Miller");
    });

    it("deletes member by id", async () => {
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
        familyId: 1,
        ownerMemberId: 2,
        type: "bank_account",
        currencyId: 1,
        bankName: "Ally Bank",
        accountName: "Savings Bucket",
        last4: "9102",
        availableBalanceCents: 2500000,
        createdAt: 1704240000,
      };

      memoryTransport.on("POST", "/api/v1/config/accounts/bank", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newBank,
      }));

      const res = await familyApi.createBankAccount({
        familyId: 1,
        ownerMemberId: 2,
        currencyId: 1,
        bankName: "Ally Bank",
        accountName: "Savings Bucket",
        last4: "9102",
        availableBalanceCents: 2500000,
      });
      expect(res.id).toBe(103);
      expect(res.type).toBe("bank_account");
      expect(res.currencyId).toBe(1);
      expect(res.bankName).toBe("Ally Bank");
      expect(res.accountName).toBe("Savings Bucket");
      expect(res.availableBalanceCents).toBe(2500000);
    });

    it("updates bank account details", async () => {
      const updated: BankAccount = {
        id: 101,
        familyId: 1,
        ownerMemberId: 1,
        type: "bank_account",
        currencyId: 1,
        bankName: "JPMorgan Chase",
        accountName: "Premier Checking",
        last4: "4821",
        availableBalanceCents: 950000,
        createdAt: 1704067200,
      };

      memoryTransport.on("PATCH", "/api/v1/config/accounts/bank/101", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updated,
      }));

      const res = await familyApi.updateBankAccount(101, {
        currencyId: 1,
        bankName: "JPMorgan Chase",
        accountName: "Premier Checking",
        last4: "4821",
        availableBalanceCents: 950000,
      });
      expect(res.id).toBe(101);
      expect(res.currencyId).toBe(1);
      expect(res.bankName).toBe("JPMorgan Chase");
      expect(res.accountName).toBe("Premier Checking");
      expect(res.availableBalanceCents).toBe(950000);
    });
  });

  describe("familyApi.createCreditCard & updateCreditCard", () => {
    it("creates credit card account and parses limits and balances in cents", async () => {
      const newCard: CreditCardAccount = {
        id: 202,
        familyId: 1,
        ownerMemberId: 1,
        type: "credit_card",
        currencyId: 1,
        bankName: "American Express",
        cardName: "Gold Card",
        last4: "1001",
        creditLimitCents: 1500000,
        availableCents: 1300000,
        outstandingCents: 200000,
        createdAt: 1704326400,
      };

      memoryTransport.on("POST", "/api/v1/config/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: newCard,
      }));

      const res = await familyApi.createCreditCard({
        familyId: 1,
        ownerMemberId: 1,
        currencyId: 1,
        bankName: "American Express",
        cardName: "Gold Card",
        last4: "1001",
        creditLimitCents: 1500000,
        availableCents: 1300000,
      });
      expect(res.id).toBe(202);
      expect(res.type).toBe("credit_card");
      expect(res.currencyId).toBe(1);
      expect(res.creditLimitCents).toBe(1500000);
      expect(res.availableCents).toBe(1300000);
      expect(res.outstandingCents).toBe(200000);
    });

    it("throws ContractViolationError if credit limit is missing or non-number", async () => {
      memoryTransport.on("POST", "/api/v1/config/accounts/credit", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: {
          id: 299,
          familyId: 1,
          ownerMemberId: 1,
          type: "credit_card",
          currencyId: 1,
          bankName: "Amex",
          cardName: "Gold",
          last4: "1001",
          creditLimitCents: "fifteen thousand", // bad type
          availableCents: 1000000,
          createdAt: 1704326400,
        },
      }));

      await expect(
        familyApi.createCreditCard({
          familyId: 1,
          ownerMemberId: 1,
          currencyId: 1,
          bankName: "Amex",
          cardName: "Gold",
          last4: "1001",
          creditLimitCents: 1500000,
          availableCents: 1000000,
        }),
      ).rejects.toThrow(ContractViolationError);
    });

    it("updates credit card details and limits", async () => {
      const updatedCard: CreditCardAccount = {
        id: 201,
        familyId: 1,
        ownerMemberId: 1,
        type: "credit_card",
        currencyId: 1,
        bankName: "Chase",
        cardName: "Sapphire Reserve",
        last4: "5561",
        creditLimitCents: 2500000,
        availableCents: 2100000,
        outstandingCents: 400000,
        createdAt: 1704153600,
      };

      memoryTransport.on("PATCH", "/api/v1/config/accounts/credit/201", () => ({
        code: Code.Zero,
        status: Status.Ok,
        data: updatedCard,
      }));

      const res = await familyApi.updateCreditCard(201, {
        currencyId: 1,
        bankName: "Chase",
        cardName: "Sapphire Reserve",
        last4: "5561",
        creditLimitCents: 2500000,
        availableCents: 2100000,
      });
      expect(res.id).toBe(201);
      expect(res.cardName).toBe("Sapphire Reserve");
      expect(res.creditLimitCents).toBe(2500000);
      expect(res.availableCents).toBe(2100000);
      expect(res.outstandingCents).toBe(400000);
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
