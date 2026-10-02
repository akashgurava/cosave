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
      expect(details.family).toBeDefined();
      expect(details.family.id).toBeTypeOf("number");
      expect(details.family.name.length).toBeGreaterThan(0);
      expect(details.family.currency.length).toBe(3);

      expect(Array.isArray(details.members)).toBe(true);
      if (details.members.length > 0) {
        expect(details.members[0]?.id).toBeTypeOf("number");
        expect(details.members[0]?.name.length).toBeGreaterThan(0);
      }

      expect(Array.isArray(details.accounts)).toBe(true);
      if (details.accounts.length > 0) {
        expect(details.accounts[0]?.id).toBeTypeOf("number");
      }
    });

    it("fetches and decodes default currency without assumption", async () => {
      const res = await familyApi.getDefaultCurrency();
      expect(res.currency).toBeDefined();
      expect(res.currency.length).toBe(3);
    });
  });

  describe("Authenticated Lifecycle Operations", () => {
    beforeAll(async () => {
      const testUser = `family_admin_${Date.now()}`;
      try {
        const regRes = await fetch(`${baseUrl}/api/v1/auth/register`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ username: testUser, password: "Password123!" }),
        });
        const cookie = regRes.headers.get("set-cookie");
        if (cookie) {
          const token = cookie.split(";")[0]?.split("=")[1];
          if (token) fetchTransport.setCookie("cosave_session", token);
        }
      } catch {
        const loginRes = await fetch(`${baseUrl}/api/v1/auth/login`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ username: "admin", password: "password123" }),
        });
        const cookie = loginRes.headers.get("set-cookie");
        if (cookie) {
          const token = cookie.split(";")[0]?.split("=")[1];
          if (token) fetchTransport.setCookie("cosave_session", token);
        }
      }
    });

    it("executes complete family, member, and account CRUD lifecycle", async () => {
      // 1. Update family display name and currency
      const updatedFamily = await familyApi.updateFamily({
        name: "The Integration Family",
        currency: "EUR",
      });
      expect(updatedFamily.name).toBe("The Integration Family");
      expect(updatedFamily.currency).toBe("EUR");
      const familyId = updatedFamily.id;

      // 2. Create member
      const member = await familyApi.createMember({
        family_id: familyId,
        name: "Alice Integration",
      });
      expect(member.id).toBeTypeOf("number");
      expect(member.name).toBe("Alice Integration");

      // 3. Update member
      const renamedMember = await familyApi.updateMember(member.id, {
        name: "Alice M. Integration",
      });
      expect(renamedMember.name).toBe("Alice M. Integration");

      // 4. Create bank account
      const bank = await familyApi.createBankAccount({
        family_id: familyId,
        owner_member_id: member.id,
        currency: "EUR",
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
        bank_name: "Nordea Bank",
        account_name: "Main Checking",
        last4: "4321",
        available_balance_cents: 350000,
        currency: "EUR",
      });
      expect(updatedBank.bank_name).toBe("Nordea Bank");
      expect(updatedBank.available_balance_cents).toBe(350000);

      // 6. Create credit card
      const card = await familyApi.createCreditCard({
        family_id: familyId,
        owner_member_id: member.id,
        currency: "GBP",
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
        bank_name: "Barclays Premier",
        card_name: "Platinum Reward Card",
        last4: "8765",
        credit_limit_cents: 700000,
        available_cents: 500000,
        currency: "GBP",
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
