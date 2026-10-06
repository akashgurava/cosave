import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { FetchTransportAdapter } from "./transport";

describe("FetchTransportAdapter", () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it("prepends baseUrl and propagates request properties", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      status: 200,
      statusText: "OK",
      headers: new Headers({ "content-type": "application/json" }),
      json: () => Promise.resolve({ success: true }),
    });
    globalThis.fetch = mockFetch;

    const adapter = new FetchTransportAdapter("http://localhost:5171");
    const res = await adapter.fetch({
      url: "/api/v1/test",
      method: "POST",
      headers: { "x-custom": "header" },
      body: JSON.stringify({ key: "val" }),
    });

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch).toHaveBeenCalledWith(
      "http://localhost:5171/api/v1/test",
      expect.objectContaining({
        method: "POST",
        headers: { "x-custom": "header" },
        body: JSON.stringify({ key: "val" }),
        credentials: "same-origin",
      }),
    );

    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ success: true });
  });

  it("stores cookies from set-cookie response header and includes them in subsequent requests", async () => {
    const mockFetch = vi
      .fn()
      .mockResolvedValueOnce({
        status: 200,
        statusText: "OK",
        headers: new Headers({ "set-cookie": "session_id=abc123xyz; Path=/; HttpOnly" }),
        json: () => Promise.resolve({}),
      })
      .mockResolvedValueOnce({
        status: 200,
        statusText: "OK",
        headers: new Headers(),
        json: () => Promise.resolve({ authenticated: true }),
      });
    globalThis.fetch = mockFetch;

    const adapter = new FetchTransportAdapter();

    // First request receives set-cookie
    await adapter.fetch({
      url: "/api/v1/auth/login",
      method: "POST",
      headers: {},
    });

    // Second request should send the cookie header
    await adapter.fetch({
      url: "/api/v1/auth/me",
      method: "GET",
      headers: {},
    });

    expect(mockFetch).toHaveBeenLastCalledWith(
      "/api/v1/auth/me",
      expect.objectContaining({
        headers: {
          cookie: "session_id=abc123xyz",
        },
      }),
    );
  });

  it("allows setting and clearing cookies manually", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      status: 200,
      statusText: "OK",
      headers: new Headers(),
      json: () => Promise.resolve({}),
    });
    globalThis.fetch = mockFetch;

    const adapter = new FetchTransportAdapter();
    adapter.setCookie("manual_token", "secret");

    await adapter.fetch({
      url: "/api/v1/resource",
      method: "GET",
      headers: {},
    });

    expect(mockFetch).toHaveBeenLastCalledWith(
      "/api/v1/resource",
      expect.objectContaining({
        headers: { cookie: "manual_token=secret" },
      }),
    );

    adapter.clearCookies();

    await adapter.fetch({
      url: "/api/v1/resource",
      method: "GET",
      headers: {},
    });

    expect(mockFetch).toHaveBeenLastCalledWith(
      "/api/v1/resource",
      expect.objectContaining({
        headers: {},
      }),
    );
  });
});
