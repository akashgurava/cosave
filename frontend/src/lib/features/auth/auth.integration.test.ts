import { describe, it, expect, beforeAll } from "vitest";
import { api, ApiError, FetchTransportAdapter } from "$lib/api";
import { authApi } from "./api";

const isIntegration =
  process.env.TEST_INTEGRATION === "1" ||
  (process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0);
const describeIntegration = isIntegration ? describe : describe.skip;

describeIntegration("Auth Live API Integration (Full-Stack Axum Roundtrip)", () => {
  const baseUrl =
    process.env.TEST_API_URL !== undefined && process.env.TEST_API_URL.length > 0
      ? process.env.TEST_API_URL
      : "http://127.0.0.1:5171";
  const fetchTransport = new FetchTransportAdapter(baseUrl);

  beforeAll(() => {
    api.setTransport(fetchTransport);
  });

  it("handles complete user authentication lifecycle against live backend", async () => {
    fetchTransport.clearCookies();

    // 1. Unauthenticated /me throws 401
    await expect(authApi.me()).rejects.toThrow(ApiError);

    // 2. Register unique user
    const username = `auth_test_${Date.now()}`;
    const password = "ValidPassword123!";
    const registered = await authApi.register({ username, password });

    expect(registered.username).toBe(username);
    expect(registered.id).toMatch(/^usr[-_]/);
    expect(registered.createdAt).toBeGreaterThan(0);

    // 3. /me succeeds with cookie issued during registration
    const meAfterRegister = await authApi.me();
    expect(meAfterRegister.id).toBe(registered.id);
    expect(meAfterRegister.username).toBe(username);

    // 4. Duplicate registration returns 409 Conflict
    await expect(authApi.register({ username, password })).rejects.toThrow(ApiError);

    // 5. Logout revokes session cookie
    await authApi.logout();
    fetchTransport.clearCookies();

    // 6. /me fails again after logout
    await expect(authApi.me()).rejects.toThrow(ApiError);

    // 7. Login with valid credentials re-establishes session
    const loggedIn = await authApi.login({ username, password });
    expect(loggedIn.id).toBe(registered.id);
    expect(loggedIn.username).toBe(username);

    // 8. /me succeeds again
    const meAfterLogin = await authApi.me();
    expect(meAfterLogin.username).toBe(username);

    // 9. Login with invalid password fails with 401
    await expect(authApi.login({ username, password: "WrongPassword!" })).rejects.toThrow(ApiError);

    // 10. Clean up session
    await authApi.logout();
    fetchTransport.clearCookies();
  });
});
