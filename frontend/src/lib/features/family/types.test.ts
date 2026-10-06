import { describe, it, expect } from "vitest";
import { ContractViolationError } from "$lib/api";
import {
  parseAccount,
  parseBankAccount,
  parseCreditCardAccount,
  parseCurrenciesResponse,
  parseCurrencyOption,
  parseDefaultCurrencyResponse,
  parseFamily,
  parseFamilyDetails,
  parseMember,
  type BankAccount,
  type CreditCardAccount,
  type Family,
  type Member,
} from "./types";

describe("Family Types & Runtime Schema Decoders", () => {
  describe("parseCurrencyOption", () => {
    it("decodes valid CurrencyOption cleanly and normalizes uppercase code", () => {
      const raw = {
        id: 1,
        code: "usd",
        name: "US Dollar",
        symbol: "$",
        scale: 2,
        sortOrder: 1,
      };
      const result = parseCurrencyOption(raw);
      expect(result.id).toBe(1);
      expect(result.code).toBe("USD");
      expect(result.name).toBe("US Dollar");
      expect(result.symbol).toBe("$");
      expect(result.scale).toBe(2);
      expect(result.sortOrder).toBe(1);
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError on invalid or missing fields", () => {
      expect(() => parseCurrencyOption(null)).toThrow(ContractViolationError);
      expect(() =>
        parseCurrencyOption({ id: "1", code: "USD", name: "Dollar", symbol: "$", scale: 2 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseCurrencyOption({ id: 1, code: "", name: "Dollar", symbol: "$", scale: 2 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseCurrencyOption({ id: 1, code: "USD", name: "", symbol: "$", scale: 2 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseCurrencyOption({ id: 1, code: "USD", name: "Dollar", symbol: "", scale: 2 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseCurrencyOption({ id: 1, code: "USD", name: "Dollar", symbol: "$", scale: -1 }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parseCurrenciesResponse", () => {
    it("decodes array of CurrencyOptions and freezes list", () => {
      const raw = [
        { id: 1, code: "USD", name: "Dollar", symbol: "$", scale: 2 },
        { id: 2, code: "EUR", name: "Euro", symbol: "€", scale: 2 },
      ];
      const result = parseCurrenciesResponse(raw);
      expect(result).toHaveLength(2);
      const firstCurr = result[0];
      const secondCurr = result[1];
      expect(firstCurr).toBeDefined();
      expect(secondCurr).toBeDefined();
      if (firstCurr !== undefined) {
        expect(firstCurr.code).toBe("USD");
      }
      if (secondCurr !== undefined) {
        expect(secondCurr.code).toBe("EUR");
      }
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError when input is not an array", () => {
      expect(() => parseCurrenciesResponse(null)).toThrow(ContractViolationError);
      expect(() => parseCurrenciesResponse({})).toThrow(ContractViolationError);
    });
  });

  describe("parseDefaultCurrencyResponse", () => {
    it("decodes valid response and normalizes currency string", () => {
      const result = parseDefaultCurrencyResponse({ currency: "inr" });
      expect(result.currency).toBe("INR");
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError when currency string is empty or missing", () => {
      expect(() => parseDefaultCurrencyResponse(null)).toThrow(ContractViolationError);
      expect(() => parseDefaultCurrencyResponse({ currency: "" })).toThrow(ContractViolationError);
      expect(() => parseDefaultCurrencyResponse({ currency: 123 })).toThrow(ContractViolationError);
    });
  });

  describe("parseFamily", () => {
    it("decodes valid Family record cleanly", () => {
      const raw = {
        id: 10,
        familyName: "  The Simpsons  ",
        currencyId: 1,
        createdAt: 1700000000,
      };
      const result: Family = parseFamily(raw);
      expect(result.id).toBe(10);
      expect(result.familyName).toBe("The Simpsons");
      expect(result.currencyId).toBe(1);
      expect(result.createdAt).toBe(1700000000);
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError on invalid fields", () => {
      expect(() => parseFamily(null)).toThrow(ContractViolationError);
      expect(() =>
        parseFamily({ id: "10", familyName: "Smith", currencyId: 1, createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() => parseFamily({ id: 10, familyName: "", currencyId: 1, createdAt: 100 })).toThrow(
        ContractViolationError,
      );
      expect(() =>
        parseFamily({ id: 10, familyName: "Smith", currencyId: "1", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseFamily({ id: 10, familyName: "Smith", currencyId: 1, createdAt: "recent" }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parseMember", () => {
    it("decodes valid Member record cleanly", () => {
      const raw = {
        id: 5,
        familyId: 10,
        memberName: " Homer ",
        createdAt: 1700000000,
      };
      const result: Member = parseMember(raw);
      expect(result.id).toBe(5);
      expect(result.familyId).toBe(10);
      expect(result.memberName).toBe("Homer");
      expect(result.createdAt).toBe(1700000000);
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError on invalid fields", () => {
      expect(() => parseMember(null)).toThrow(ContractViolationError);
      expect(() =>
        parseMember({ id: "5", familyId: 10, memberName: "Homer", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseMember({ id: 5, familyId: "10", memberName: "Homer", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() => parseMember({ id: 5, familyId: 10, memberName: "", createdAt: 100 })).toThrow(
        ContractViolationError,
      );
    });
  });

  describe("parseBankAccount", () => {
    it("decodes valid BankAccount record cleanly", () => {
      const raw = {
        type: "bank_account",
        id: 101,
        familyId: 10,
        ownerMemberId: 5,
        currencyId: 1,
        bankName: "First Bank",
        accountName: "Checking",
        last4: "1234",
        availableBalance: 50000,
        createdAt: 1700000000,
      };
      const result: BankAccount = parseBankAccount(raw);
      expect(result.type).toBe("bank_account");
      expect(result.id).toBe(101);
      expect(result.bankName).toBe("First Bank");
      expect(result.accountName).toBe("Checking");
      expect(result.last4).toBe("1234");
      expect(result.availableBalance).toBe(50000);
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError on discriminator or field errors", () => {
      expect(() => parseBankAccount({ type: "credit_card" })).toThrow(ContractViolationError);
      expect(() =>
        parseBankAccount({
          type: "bank_account",
          id: 101,
          familyId: 10,
          ownerMemberId: 5,
          currencyId: 1,
          bankName: "Bank",
          accountName: "Checking",
          last4: "123", // Not 4 chars
          availableBalance: 5000,
          createdAt: 100,
        }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseBankAccount({
          type: "bank_account",
          id: 101,
          familyId: 10,
          ownerMemberId: 5,
          currencyId: 1,
          bankName: "Bank",
          accountName: "Checking",
          last4: "1234",
          availableBalance: 50.5, // Non-integer
          createdAt: 100,
        }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parseCreditCardAccount", () => {
    it("decodes valid CreditCardAccount record cleanly", () => {
      const raw = {
        type: "credit_card",
        id: 201,
        familyId: 10,
        ownerMemberId: 5,
        currencyId: 1,
        bankName: "Chase",
        cardName: "Sapphire",
        last4: "9876",
        creditLimit: 100000,
        availableCredit: 80000,
        outstandingBalance: 20000,
        createdAt: 1700000000,
      };
      const result: CreditCardAccount = parseCreditCardAccount(raw);
      expect(result.type).toBe("credit_card");
      expect(result.id).toBe(201);
      expect(result.cardName).toBe("Sapphire");
      expect(result.last4).toBe("9876");
      expect(result.creditLimit).toBe(100000);
      expect(result.availableCredit).toBe(80000);
      expect(result.outstandingBalance).toBe(20000);
      expect(Object.isFrozen(result)).toBe(true);
    });

    it("throws ContractViolationError on negative credit limit or invalid balance", () => {
      expect(() =>
        parseCreditCardAccount({
          type: "credit_card",
          id: 201,
          familyId: 10,
          ownerMemberId: 5,
          currencyId: 1,
          bankName: "Chase",
          cardName: "Card",
          last4: "1234",
          creditLimit: -500, // Negative limit
          availableCredit: 1000,
          outstandingBalance: 0,
          createdAt: 100,
        }),
      ).toThrow(ContractViolationError);
    });
  });

  describe("parseAccount (Discriminated Union)", () => {
    it("delegates bank_account correctly", () => {
      const raw = {
        type: "bank_account",
        id: 1,
        familyId: 1,
        ownerMemberId: 1,
        currencyId: 1,
        bankName: "Bank",
        accountName: "Checking",
        last4: "1111",
        availableBalance: 100,
        createdAt: 100,
      };
      const account = parseAccount(raw);
      expect(account.type).toBe("bank_account");
    });

    it("delegates credit_card correctly", () => {
      const raw = {
        type: "credit_card",
        id: 2,
        familyId: 1,
        ownerMemberId: 1,
        currencyId: 1,
        bankName: "Bank",
        cardName: "Card",
        last4: "2222",
        creditLimit: 1000,
        availableCredit: 800,
        outstandingBalance: 200,
        createdAt: 100,
      };
      const account = parseAccount(raw);
      expect(account.type).toBe("credit_card");
    });

    it("throws ContractViolationError on unknown account type discriminator", () => {
      expect(() => parseAccount({ type: "crypto_wallet" })).toThrow(ContractViolationError);
    });
  });

  describe("parseFamilyDetails", () => {
    it("decodes complete family details payload with nested models", () => {
      const raw = {
        family: {
          id: 1,
          familyName: "Household",
          currencyId: 1,
          createdAt: 100,
        },
        members: [{ id: 1, familyId: 1, memberName: "Alice", createdAt: 100 }],
        accounts: [
          {
            type: "bank_account",
            id: 10,
            familyId: 1,
            ownerMemberId: 1,
            currencyId: 1,
            bankName: "Nordea",
            accountName: "Checking",
            last4: "4444",
            availableBalance: 15000,
            createdAt: 100,
          },
        ],
        currencies: [{ id: 1, code: "USD", name: "Dollar", symbol: "$", scale: 2 }],
      };
      const result = parseFamilyDetails(raw);
      expect(result.family).not.toBeNull();
      if (result.family !== null) {
        expect(result.family.familyName).toBe("Household");
      }
      expect(result.members).toHaveLength(1);
      expect(result.accounts).toHaveLength(1);
      expect(result.currencies).toHaveLength(1);
      expect(Object.isFrozen(result)).toBe(true);
      expect(Object.isFrozen(result.members)).toBe(true);
      expect(Object.isFrozen(result.accounts)).toBe(true);
      expect(Object.isFrozen(result.currencies)).toBe(true);
    });

    it("allows null family for fresh initial state", () => {
      const raw = {
        family: null,
        members: [],
        accounts: [],
        currencies: [],
      };
      const result = parseFamilyDetails(raw);
      expect(result.family).toBeNull();
      expect(result.members).toEqual([]);
    });

    it("throws ContractViolationError when arrays are invalid", () => {
      expect(() =>
        parseFamilyDetails({ family: null, members: "invalid", accounts: [], currencies: [] }),
      ).toThrow(ContractViolationError);
    });
  });
});
