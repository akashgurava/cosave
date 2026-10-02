import { describe, it, expect, beforeAll } from "vitest";
import { api, FetchTransportAdapter } from "$lib/api";
import { familyApi } from "./api";

const isIntegration = process.env.TEST_INTEGRATION === "1" || !!process.env.TEST_API_URL;
const describeIntegration = isIntegration ? describe : describe.skip;

describeIntegration("Family Live API Integration (Full-Stack Axum Roundtrip)", () => {
  const baseUrl = process.env.TEST_API_URL || "http://127.0.0.1:5171";
  const fetchTransport = new FetchTransportAdapter(baseUrl);

  beforeAll(() => {
    api.setTransport(fetchTransport);
  });

  describe("Read Operations", () => {
    it("fetches and decodes real family details from live backend", async () => {
      const details = await familyApi.getDetails();
      if (details.family !== null) {
        expect(details.family.id).toBeTypeOf("number");
        expect(details.family.family_name.length).toBeGreaterThan(0);
        expect(details.family.currency_id).toBeTypeOf("number");
      }

      expect(Array.isArray(details.members)).toBe(true);
      if (details.members.length > 0) {
        expect(details.members[0]?.id).toBeTypeOf("number");
        expect(details.members[0]?.member_name.length).toBeGreaterThan(0);
      }

      expect(Array.isArray(details.accounts)).toBe(true);
      if (details.accounts.length > 0) {
        expect(details.accounts[0]?.id).toBeTypeOf("number");
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
      expect(inRes.currency).toBe("INR");
    });
  });

  describe("Authenticated Lifecycle Operations", () => {
    beforeAll(async () => {
      api.setTransport(fetchTransport);
      const testUser = `family_admin_${Date.now()}`;
      let token: string | undefined;

      const regRes = await fetch(`${baseUrl}/api/v1/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: testUser, password: "Password123!" }),
      });

      if (regRes.ok) {
        const cookie = regRes.headers.get("set-cookie");
        token = cookie?.match(/cosave_session=([^;]+)/)?.[1];
      }

      if (!token) {
        const loginRes = await fetch(`${baseUrl}/api/v1/auth/login`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ username: "admin", password: "password123" }),
        });
        const cookie = loginRes.headers.get("set-cookie");
        token = cookie?.match(/cosave_session=([^;]+)/)?.[1];
      }

      if (token) {
        fetchTransport.setCookie("cosave_session", token);
      } else {
        throw new Error(
          `Failed to authenticate for integration tests: register status ${regRes.status}`,
        );
      }
    });

    it("executes complete family, member, and account CRUD lifecycle", async () => {
      const currencies = await familyApi.getCurrencies();
      const eur = currencies.find((c) => c.code === "EUR") ?? currencies[0]!;
      const gbp = currencies.find((c) => c.code === "GBP") ?? currencies[0]!;

      // 1. Update family display name and currency
      const updatedFamily = await familyApi.updateFamily({
        family_name: "The Integration Family",
        currency_id: eur.id,
      });
      expect(updatedFamily.family_name).toBe("The Integration Family");
      expect(updatedFamily.currency_id).toBe(eur.id);
      const familyId = updatedFamily.id;

      // 2. Create member
      const member = await familyApi.createMember({
        family_id: familyId,
        member_name: "Alice Integration",
      });
      expect(member.id).toBeTypeOf("number");
      expect(member.member_name).toBe("Alice Integration");

      // 3. Update member
      const renamedMember = await familyApi.updateMember(member.id, {
        member_name: "Alice M. Integration",
      });
      expect(renamedMember.member_name).toBe("Alice M. Integration");

      // 4. Create bank account
      const bank = await familyApi.createBankAccount({
        family_id: familyId,
        owner_member_id: member.id,
        currency_id: eur.id,
        bank_name: "Nordea",
        account_name: "Checking",
        last4: "4321",
        available_balance_cents: 250000,
      });
      expect(bank.id).toBeTypeOf("number");
      expect(bank.bank_name).toBe("Nordea");
      expect(bank.type).toBe("bank_account");
      expect(bank.available_balance_cents).toBe(250000);

      // 5. Update bank account
      const updatedBank = await familyApi.updateBankAccount(bank.id, {
        currency_id: eur.id,
        bank_name: "Nordea Bank",
        account_name: "Main Checking",
        last4: "4321",
        available_balance_cents: 350000,
      });
      expect(updatedBank.bank_name).toBe("Nordea Bank");
      expect(updatedBank.available_balance_cents).toBe(350000);

      // 6. Create credit card
      const card = await familyApi.createCreditCard({
        family_id: familyId,
        owner_member_id: member.id,
        currency_id: gbp.id,
        bank_name: "Barclays",
        card_name: "Reward Card",
        last4: "8765",
        credit_limit_cents: 500000,
        available_cents: 400000,
      });
      expect(card.id).toBeTypeOf("number");
      expect(card.type).toBe("credit_card");
      expect(card.outstanding_cents).toBe(100000);

      // 7. Update credit card
      const updatedCard = await familyApi.updateCreditCard(card.id, {
        currency_id: gbp.id,
        bank_name: "Barclays Premier",
        card_name: "Platinum Reward Card",
        last4: "8765",
        credit_limit_cents: 700000,
        available_cents: 500000,
      });
      expect(updatedCard.bank_name).toBe("Barclays Premier");
      expect(updatedCard.outstanding_cents).toBe(200000);

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
