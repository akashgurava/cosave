import { describe, it, expect, beforeAll } from "vitest";
import { api, FetchTransportAdapter } from "$lib/api";
import {
  toAccountId,
  toCategoryId,
  toMinorUnits,
  toTypeId,
  type AccountId,
  type CategoryId,
  type TypeId,
} from "$lib/types";
import { familyApi } from "../family/api";
import { categoriesApi } from "../categories/api";
import { transactionsApi } from "./api";

const isIntegration =
  process.env.TEST_INTEGRATION === "1" ||
  (process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0);
const describeIntegration = isIntegration ? describe : describe.skip;

describeIntegration("Transactions Live API Integration (Full-Stack Axum Roundtrip)", () => {
  const baseUrl =
    process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0
      ? process.env.TEST_API_URL
      : "http://127.0.0.1:5171";
  const fetchTransport = new FetchTransportAdapter(baseUrl);

  let accountId: AccountId;
  let typeId: TypeId;
  let categoryId: CategoryId;

  beforeAll(async () => {
    api.setTransport(fetchTransport);
    const testUser = `tx_admin_${Date.now()}`;
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

    // Seed test prerequisites: family, member, account, and hierarchy
    const details = await familyApi.getDetails();
    let family = details.family;
    if (family === null) {
      const currencies = await familyApi.getCurrencies();
      const firstCurrency = currencies[0];
      if (firstCurrency === undefined) {
        throw new Error("No currencies found on backend");
      }
      family = await familyApi.createFamily({
        familyName: "Transaction Integration Family",
        currencyId: firstCurrency.id,
      });
    }

    let member = details.members[0];
    if (member === undefined) {
      member = await familyApi.createMember({
        familyId: family.id,
        memberName: "Transaction Tester",
      });
    }

    let account = details.accounts[0];
    if (account === undefined) {
      account = await familyApi.createBankAccount({
        familyId: family.id,
        ownerMemberId: member.id,
        currencyId: family.currencyId,
        bankName: "Test Bank",
        accountName: "Integration Checking",
        last4: "9999",
        availableBalance: toMinorUnits(100000),
      });
    }
    accountId = toAccountId(account.id);

    const hierarchy = await categoriesApi.getHierarchy();
    const firstType = hierarchy.types[0];
    if (firstType === undefined) {
      throw new Error("No types available on backend hierarchy");
    }
    const firstCat = firstType.categories[0];
    if (firstCat === undefined) {
      throw new Error("No categories available on backend hierarchy");
    }
    typeId = toTypeId(firstType.id);
    categoryId = toCategoryId(firstCat.id);
  });

  describe("Read Operations", () => {
    it("fetches paginated transactions from live backend", async () => {
      const res = await transactionsApi.getTransactions({ page: 1, pageSize: 20 });
      expect(res.page).toBe(1);
      expect(res.pageSize).toBe(20);
      expect(res.totalCount).toBeGreaterThanOrEqual(0);
      expect(res.totalPages).toBeGreaterThanOrEqual(1);
      expect(Array.isArray(res.items)).toBe(true);
    });
  });

  describe("Authenticated Lifecycle Operations", () => {
    it("executes complete transaction CRUD lifecycle with filter and search validation", async () => {
      const uniqueSuffix = Date.now();
      const initialDesc = `Supermarket Purchase ${uniqueSuffix}`;
      const updatedDesc = `Supermarket Purchase Premium ${uniqueSuffix}`;

      // 1. Create transaction via POST /api/v1/transactions
      const created = await transactionsApi.createTransaction({
        source: "manual",
        date: "2026-10-10",
        description: initialDesc,
        payee: "Fresh Market",
        amount: toMinorUnits(-3250),
        typeId,
        accountId,
        categoryId,
        status: "cleared",
      });

      expect(typeof created.id).toBe("string");
      expect(created.id.length).toBeGreaterThan(0);
      expect(created.source).toBe("manual");
      expect(created.description).toBe(initialDesc);
      expect(created.payee).toBe("Fresh Market");
      expect(created.amount).toBe(-3250);
      expect(created.date).toBe("2026-10-10");
      expect(created.typeId).toBe(typeId);
      expect(created.accountId).toBe(accountId);
      expect(created.categoryId).toBe(categoryId);
      expect(created.status).toBe("cleared");

      const createdId = created.id;

      // 2. Fetch created transaction by ID via GET /api/v1/transactions/:id
      const fetched = await transactionsApi.getTransaction(createdId);
      expect(fetched.id).toBe(createdId);
      expect(fetched.description).toBe(initialDesc);
      expect(fetched.amount).toBe(-3250);

      // 3. Search transactions by query
      const searchRes = await transactionsApi.getTransactions({ query: initialDesc });
      expect(searchRes.items.some((item) => item.id === createdId)).toBe(true);

      // 4. Update transaction via PATCH /api/v1/transactions/:id
      const updated = await transactionsApi.updateTransaction({
        ...created,
        description: updatedDesc,
        payee: "Fresh Market Organic",
        amount: toMinorUnits(-4200),
      });

      expect(updated.id).toBe(createdId);
      expect(updated.description).toBe(updatedDesc);
      expect(updated.payee).toBe("Fresh Market Organic");
      expect(updated.amount).toBe(-4200);

      // 5. Delete transaction via DELETE /api/v1/transactions/:id with source body
      const delResult = await transactionsApi.deleteTransaction(createdId, "manual");
      expect(delResult).toBeNull();

      // 6. Verify deleted transaction returns 404 or fails to fetch
      await expect(transactionsApi.getTransaction(createdId)).rejects.toThrow();
    });

    it("creates and fetches a transaction with null description against live backend", async () => {
      const created = await transactionsApi.createTransaction({
        source: "manual",
        date: "2026-10-10",
        description: null,
        payee: "Gas Station",
        amount: toMinorUnits(-5000),
        typeId,
        accountId,
        categoryId,
        status: "cleared",
      });

      expect(created.description).toBeNull();
      const fetched = await transactionsApi.getTransaction(created.id);
      expect(fetched.description).toBeNull();

      await transactionsApi.deleteTransaction(created.id, "manual");
    });
  });
});
