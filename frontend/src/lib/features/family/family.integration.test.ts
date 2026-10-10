import { describe, it, expect, beforeAll } from "vitest";
import { api, FetchTransportAdapter } from "$lib/api";
import { toMinorUnits } from "$lib/types";
import { familyApi } from "./api";

const isIntegration =
  process.env.TEST_INTEGRATION === "1" ||
  (process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0);
const describeIntegration = isIntegration ? describe : describe.skip;

describeIntegration("Family Live API Integration (Full-Stack Axum Roundtrip)", () => {
  const baseUrl =
    process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0
      ? process.env.TEST_API_URL
      : "http://127.0.0.1:5171";
  const fetchTransport = new FetchTransportAdapter(baseUrl);

  beforeAll(async () => {
    api.setTransport(fetchTransport);
    const testUser = `family_admin_${Date.now()}`;
    let token: string | undefined;

    const regRes = await fetch(`${baseUrl}/api/v1/auth/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username: testUser, password: "Password123!" }),
    });

    if (regRes.ok === true) {
      const cookie = regRes.headers.get("set-cookie");
      if (cookie !== null) {
        const match = cookie.match(/cosave_session=([^;]+)/);
        if (match !== null && match[1] !== undefined) {
          token = match[1];
        }
      }
    }

    if (token === undefined) {
      const loginRes = await fetch(`${baseUrl}/api/v1/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: "admin", password: "password123" }),
      });
      const cookie = loginRes.headers.get("set-cookie");
      if (cookie !== null) {
        const match = cookie.match(/cosave_session=([^;]+)/);
        if (match !== null && match[1] !== undefined) {
          token = match[1];
        }
      }
    }

    if (token !== undefined) {
      fetchTransport.setCookie("cosave_session", token);
    } else {
      throw new Error(
        `Failed to authenticate for integration tests: register status ${regRes.status}`,
      );
    }
  });

  describe("Read Operations", () => {
    it("fetches and decodes real family details from live backend", async () => {
      const details = await familyApi.getDetails();
      if (details.family !== null) {
        expect(details.family.id).toBeTypeOf("number");
        expect(details.family.familyName.length).toBeGreaterThan(0);
        expect(details.family.currencyId).toBeTypeOf("number");
      }

      expect(Array.isArray(details.members)).toBe(true);
      if (details.members.length > 0) {
        const firstMem = details.members[0];
        expect(firstMem).toBeDefined();
        if (firstMem !== undefined) {
          expect(firstMem.id).toBeTypeOf("number");
          expect(firstMem.memberName.length).toBeGreaterThan(0);
        }
      }

      expect(Array.isArray(details.accounts)).toBe(true);
      if (details.accounts.length > 0) {
        const firstAcc = details.accounts[0];
        expect(firstAcc).toBeDefined();
        if (firstAcc !== undefined) {
          expect(firstAcc.id).toBeTypeOf("number");
        }
      }

      expect(Array.isArray(details.currencies)).toBe(true);
      expect(details.currencies.length).toBeGreaterThanOrEqual(20);
      expect(details.currencies.some((c) => c.code === "USD")).toBe(true);
      expect(details.currencies.some((c) => c.code === "INR")).toBe(true);
    });

    it("fetches supported currencies list from live backend", async () => {
      const currencies = await familyApi.getCurrencies();
      expect(Array.isArray(currencies)).toBe(true);
      expect(currencies.length).toBeGreaterThanOrEqual(20);
      expect(currencies.some((c) => c.code === "EUR")).toBe(true);
    });

    it("fetches and decodes default currency without assumption", async () => {
      const res = await familyApi.getDefaultCurrency();
      expect(res.currency).toBeDefined();
      expect(res.currency.length).toBe(3);

      const inRes = await familyApi.getDefaultCurrency("IN");
      expect(inRes.currency).toBeDefined();
      expect(inRes.currency.length).toBe(3);
    });
  });

  describe("Authenticated Lifecycle Operations", () => {
    it("executes complete family, member, and account CRUD lifecycle", async () => {
      const currencies = await familyApi.getCurrencies();
      const fallback = currencies[0];
      if (fallback === undefined) {
        throw new Error("No currencies found on backend");
      }
      const foundEur = currencies.find((c) => c.code === "EUR");
      const eur = foundEur !== undefined ? foundEur : fallback;
      const foundGbp = currencies.find((c) => c.code === "GBP");
      const gbp = foundGbp !== undefined ? foundGbp : fallback;

      // 1. Update family display name and currency
      const updatedFamily = await familyApi.updateFamily({
        familyName: "The Integration Family",
        currencyId: eur.id,
      });
      expect(updatedFamily.familyName).toBe("The Integration Family");
      expect(updatedFamily.currencyId).toBe(eur.id);
      const familyId = updatedFamily.id;

      // 2. Create member
      const member = await familyApi.createMember({
        familyId,
        memberName: "Alice Integration",
      });
      expect(member.id).toBeTypeOf("number");
      expect(member.memberName).toBe("Alice Integration");

      // 3. Update member
      const renamedMember = await familyApi.updateMember(member.id, {
        memberName: "Alice M. Integration",
      });
      expect(renamedMember.memberName).toBe("Alice M. Integration");

      // 4. Create bank account
      const bank = await familyApi.createBankAccount({
        familyId,
        ownerMemberId: member.id,
        currencyId: eur.id,
        bankName: "Nordea",
        accountName: "Checking",
        last4: "4321",
        availableBalance: toMinorUnits(250000),
      });
      expect(bank.id).toBeTypeOf("number");
      expect(bank.bankName).toBe("Nordea");
      expect(bank.type).toBe("bank_account");
      expect(bank.availableBalance).toBe(250000);

      // 5. Update bank account
      const updatedBank = await familyApi.updateBankAccount(bank.id, {
        currencyId: eur.id,
        bankName: "Nordea Bank",
        accountName: "Main Checking",
        last4: "4321",
        availableBalance: toMinorUnits(350000),
      });
      expect(updatedBank.bankName).toBe("Nordea Bank");
      expect(updatedBank.availableBalance).toBe(350000);

      // 5b. Verify currency mismatch rejection on live backend
      await expect(
        familyApi.createCreditCard({
          familyId,
          ownerMemberId: member.id,
          currencyId: gbp.id,
          bankName: "Barclays",
          cardName: "Reward Card",
          last4: "8765",
          creditLimit: toMinorUnits(500000),
          availableCredit: toMinorUnits(400000),
        }),
      ).rejects.toThrow("Account currency ID");

      // 6. Create credit card (defaults to household base currency EUR when omitted or matching)
      const card = await familyApi.createCreditCard({
        familyId,
        ownerMemberId: member.id,
        bankName: "Barclays",
        cardName: "Reward Card",
        last4: "8765",
        creditLimit: toMinorUnits(500000),
        availableCredit: toMinorUnits(400000),
      });
      expect(card.id).toBeTypeOf("number");
      expect(card.type).toBe("credit_card");
      expect(card.outstandingBalance).toBe(100000);

      // 7. Update credit card
      const updatedCard = await familyApi.updateCreditCard(card.id, {
        currencyId: eur.id,
        bankName: "Barclays Premier",
        cardName: "Platinum Reward Card",
        last4: "8765",
        creditLimit: toMinorUnits(700000),
        availableCredit: toMinorUnits(500000),
      });
      expect(updatedCard.bankName).toBe("Barclays Premier");
      expect(updatedCard.outstandingBalance).toBe(200000);

      // 8. Delete accounts
      const delCardRes = await familyApi.deleteAccount(card.id);
      expect(delCardRes).toBeNull();
      const delBankRes = await familyApi.deleteAccount(bank.id);
      expect(delBankRes).toBeNull();

      // 9. Delete member
      const delMemRes = await familyApi.deleteMember(member.id);
      expect(delMemRes).toBeNull();
    });
  });
});
