/**
 * Pluggable HTTP transport adapters for browser requests and integration tests.
 *
 * Provides a uniform interface over fetch, including cookie jar tracking
 * for full-stack integration test flows.
 */

export interface TransportRequest {
  url: string;
  method: "GET" | "POST" | "PATCH" | "DELETE";
  headers: Record<string, string>;
  body?: string;
  signal?: AbortSignal;
}

export interface TransportResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  json(): Promise<unknown>;
}

export interface TransportAdapter {
  fetch(req: TransportRequest): Promise<TransportResponse>;
}

export class FetchTransportAdapter implements TransportAdapter {
  private cookies = new Map<string, string>();

  constructor(private readonly baseUrl: string = "") {}

  setCookie(name: string, value: string): void {
    this.cookies.set(name, value);
  }

  clearCookies(): void {
    this.cookies.clear();
  }

  async fetch(req: TransportRequest): Promise<TransportResponse> {
    const fetchFn = typeof window !== "undefined" ? window.fetch : globalThis.fetch;
    const url = this.baseUrl.length > 0 ? `${this.baseUrl}${req.url}` : req.url;
    const headers: Record<string, string> = { ...req.headers };

    if (
      this.cookies.size > 0 &&
      headers["cookie"] === undefined &&
      headers["Cookie"] === undefined
    ) {
      const cookieStr = Array.from(this.cookies.entries())
        .map(([k, v]) => `${k}=${v}`)
        .join("; ");
      headers["cookie"] = cookieStr;
    }

    const res = await fetchFn(url, {
      method: req.method,
      headers,
      body: req.body,
      signal: req.signal,
      credentials: "same-origin",
    });

    const setCookie = res.headers.get("set-cookie");
    if (setCookie !== null && setCookie.length > 0) {
      const firstSegment = setCookie.split(";")[0];
      const parts = firstSegment !== undefined ? firstSegment.trim() : "";
      if (parts.length > 0) {
        const eqIdx = parts.indexOf("=");
        if (eqIdx !== -1) {
          const k = parts.slice(0, eqIdx).trim();
          const v = parts.slice(eqIdx + 1).trim();
          this.cookies.set(k, v);
        }
      }
    }

    return {
      status: res.status,
      statusText: res.statusText,
      headers: Object.fromEntries(res.headers.entries()),
      json: () => res.json(),
    };
  }
}
