import { describe, it, expect } from "vitest";
import { MemoryTransportAdapter } from "./testing";

describe("MemoryTransportAdapter", () => {
  it("returns 404 for unregistered endpoint", async () => {
    const adapter = new MemoryTransportAdapter();
    const res = await adapter.fetch({
      url: "/api/v1/missing",
      method: "GET",
      headers: {},
    });

    expect(res.status).toBe(404);
    expect(res.statusText).toBe("Not Found");
    const json = (await res.json()) as { code: number; status: string; data: { error: string } };
    expect(json.code).toBe(404);
    expect(json.status).toBe("NOT_FOUND");
    expect(json.data.error).toContain("No mock registered for GET /api/v1/missing");
  });

  it("wraps raw payload in standard { code: 0, status: 'OK', data } envelope if not already enveloped", async () => {
    const adapter = new MemoryTransportAdapter();
    adapter.on("GET", "/api/v1/raw", () => ({ count: 42 }));

    const res = await adapter.fetch({
      url: "/api/v1/raw",
      method: "GET",
      headers: {},
    });

    expect(res.status).toBe(200);
    expect(res.statusText).toBe("OK");
    const json = (await res.json()) as { code: number; status: string; data: { count: number } };
    expect(json.code).toBe(0);
    expect(json.status).toBe("OK");
    expect(json.data.count).toBe(42);
  });

  it("preserves explicit error envelopes and maps status code", async () => {
    const adapter = new MemoryTransportAdapter();
    adapter.on("POST", "/api/v1/fail", () => ({
      code: 400,
      status: "BAD_REQUEST",
      data: { message: "Invalid input" },
    }));

    const res = await adapter.fetch({
      url: "/api/v1/fail",
      method: "POST",
      headers: {},
    });

    expect(res.status).toBe(400);
    expect(res.statusText).toBe("BAD_REQUEST");
    const json = (await res.json()) as { code: number; status: string };
    expect(json.code).toBe(400);
    expect(json.status).toBe("BAD_REQUEST");
  });

  it("strips query parameters when matching registered endpoints", async () => {
    const adapter = new MemoryTransportAdapter();
    adapter.on("GET", "/api/v1/items", (req) => {
      expect(req.url).toBe("/api/v1/items?page=1&limit=10");
      return { items: [] };
    });

    const res = await adapter.fetch({
      url: "/api/v1/items?page=1&limit=10",
      method: "GET",
      headers: {},
    });

    expect(res.status).toBe(200);
    const json = (await res.json()) as { code: number; data: { items: unknown[] } };
    expect(json.code).toBe(0);
  });
});
