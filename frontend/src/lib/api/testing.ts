/**
 * Test utilities and mock transport adapters for frontend unit and contract tests.
 *
 * Provides `MemoryTransportAdapter` for deterministic in-memory mocking of backend
 * REST endpoints without network overhead or real HTTP servers.
 */

import type { TransportAdapter, TransportRequest, TransportResponse } from "./transport";

/**
 * In-memory mock transport adapter for Tier 2 contract tests.
 * Enables zero-network, synchronous-like mocking of backend endpoints.
 */
export class MemoryTransportAdapter implements TransportAdapter {
  private handlers = new Map<string, (req: TransportRequest) => Promise<unknown> | unknown>();

  on(method: string, pathPattern: string, handler: (req: TransportRequest) => unknown): this {
    this.handlers.set(`${method.toUpperCase()} ${pathPattern}`, handler);
    return this;
  }

  async fetch(req: TransportRequest): Promise<TransportResponse> {
    const firstUrlSegment = req.url.split("?")[0];
    const cleanUrl = firstUrlSegment !== undefined ? firstUrlSegment : req.url;
    const key = `${req.method.toUpperCase()} ${cleanUrl}`;
    const handler = this.handlers.get(key);

    if (handler === undefined) {
      return {
        status: 404,
        statusText: "Not Found",
        headers: { "content-type": "application/json" },
        json: async () => ({
          code: 404,
          status: "NOT_FOUND",
          data: { error: `No mock registered for ${key}` },
        }),
      };
    }

    const result = await handler(req);
    const isEnvelope =
      typeof result === "object" && result !== null && "code" in result && "status" in result;
    const envelope = isEnvelope ? result : { code: 0, status: "OK", data: result };
    const code = (envelope as { code: number }).code;
    const statusStr = (envelope as { status: string }).status;
    const statusText = statusStr !== undefined && statusStr.length > 0 ? statusStr : "OK";

    return {
      status: code !== 0 && code >= 400 ? code : 200,
      statusText,
      headers: { "content-type": "application/json" },
      json: async () => envelope,
    };
  }
}
