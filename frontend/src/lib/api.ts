/**
 * Canonical status codes and envelope types matching the Rust Axum backend.
 */
export const Code = {
  Zero: 0,
  BadRequest: 400,
  Unauthorized: 401,
  Conflict: 409,
  InternalError: 500,
  zero: (): 0 => 0,
  badRequest: (): 400 => 400,
  unauthorized: (): 401 => 401,
  conflict: (): 409 => 409,
  internalError: (): 500 => 500,
} as const;

export type Code = 0 | 400 | 401 | 409 | 500;

export const Status = {
  Healthy: "HEALTHY",
  Ok: "OK",
  BadRequest: "BAD_REQUEST",
  Unauthenticated: "UNAUTHENTICATED",
  InvalidCredentials: "INVALID_CREDENTIALS",
  UserExists: "USER_EXISTS",
  InternalError: "INTERNAL_ERROR",
  healthy: (): "HEALTHY" => "HEALTHY",
  ok: (): "OK" => "OK",
  badRequest: (): "BAD_REQUEST" => "BAD_REQUEST",
  unauthenticated: (): "UNAUTHENTICATED" => "UNAUTHENTICATED",
  invalidCredentials: (): "INVALID_CREDENTIALS" => "INVALID_CREDENTIALS",
  userExists: (): "USER_EXISTS" => "USER_EXISTS",
  internalError: (): "INTERNAL_ERROR" => "INTERNAL_ERROR",
} as const;

export type Status =
  | "HEALTHY"
  | "OK"
  | "BAD_REQUEST"
  | "UNAUTHENTICATED"
  | "INVALID_CREDENTIALS"
  | "USER_EXISTS"
  | "INTERNAL_ERROR";

export interface ApiResponse<T> {
  code: Code;
  status: Status;
  data: T;
}

/**
 * Unified error class for all failures crossing the network/contract seam.
 */
export class ApiError extends Error {
  public override readonly name: string = "ApiError";

  constructor(
    message: string,
    public readonly httpStatus: number = 0,
    public readonly code: Code | number = 0,
    public readonly apiStatus: Status | string = "ERROR",
    public readonly details: unknown = null,
  ) {
    super(message);
  }

  get isUnauthorized(): boolean {
    return (
      this.httpStatus === 401 ||
      this.code === 401 ||
      this.apiStatus === "UNAUTHENTICATED" ||
      this.apiStatus === "INVALID_CREDENTIALS"
    );
  }

  get isNotFound(): boolean {
    return this.httpStatus === 404;
  }

  get isConflict(): boolean {
    return this.httpStatus === 409 || this.code === 409 || this.apiStatus === "USER_EXISTS";
  }
}

export class UnanticipatedCodeError extends ApiError {
  public override readonly name: string = "UnanticipatedCodeError";
  constructor(public readonly rawCode: unknown) {
    super(
      `Unanticipated API response code: ${JSON.stringify(rawCode)}`,
      500,
      500,
      "INTERNAL_ERROR",
    );
  }
}

export class UnanticipatedStatusError extends ApiError {
  public override readonly name: string = "UnanticipatedStatusError";
  constructor(public readonly rawStatus: unknown) {
    super(
      `Unanticipated API response status: ${JSON.stringify(rawStatus)}`,
      500,
      500,
      "INTERNAL_ERROR",
    );
  }
}

export function parseCode(rawCode: unknown): Code {
  if (rawCode === Code.Zero || rawCode === 0) return Code.Zero;
  if (rawCode === Code.BadRequest || rawCode === 400) return Code.BadRequest;
  if (rawCode === Code.Unauthorized || rawCode === 401) return Code.Unauthorized;
  if (rawCode === Code.Conflict || rawCode === 409) return Code.Conflict;
  if (rawCode === Code.InternalError || rawCode === 500) return Code.InternalError;
  throw new UnanticipatedCodeError(rawCode);
}

export function parseStatus(rawStatus: unknown): Status {
  if (rawStatus === Status.Healthy || rawStatus === "HEALTHY") return Status.Healthy;
  if (rawStatus === Status.Ok || rawStatus === "OK") return Status.Ok;
  if (rawStatus === Status.BadRequest || rawStatus === "BAD_REQUEST") return Status.BadRequest;
  if (rawStatus === Status.Unauthenticated || rawStatus === "UNAUTHENTICATED")
    return Status.Unauthenticated;
  if (rawStatus === Status.InvalidCredentials || rawStatus === "INVALID_CREDENTIALS")
    return Status.InvalidCredentials;
  if (rawStatus === Status.UserExists || rawStatus === "USER_EXISTS") return Status.UserExists;
  if (rawStatus === Status.InternalError || rawStatus === "INTERNAL_ERROR")
    return Status.InternalError;
  throw new UnanticipatedStatusError(rawStatus);
}

export class ContractViolationError extends ApiError {
  public override readonly name: string = "ContractViolationError";
  constructor(message: string, details: unknown = null) {
    super(`API Contract Violation: ${message}`, 200, 0, "CONTRACT_VIOLATION", details);
  }
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function parseNull(raw: unknown): null {
  if (raw === null || raw === undefined) {
    return null;
  }
  throw new ContractViolationError(`Expected null response data, got: ${JSON.stringify(raw)}`);
}

/**
 * Category 3 Transport Seam (Ports & Adapters)
 */
export interface TransportRequest {
  url: string;
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
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
  async fetch(req: TransportRequest): Promise<TransportResponse> {
    const res = await window.fetch(req.url, {
      method: req.method,
      headers: req.headers,
      body: req.body,
      signal: req.signal,
      credentials: "same-origin",
    });
    return {
      status: res.status,
      statusText: res.statusText,
      headers: Object.fromEntries(res.headers.entries()),
      json: () => res.json(),
    };
  }
}

export class MemoryTransportAdapter implements TransportAdapter {
  private handlers = new Map<string, (req: TransportRequest) => Promise<unknown> | unknown>();

  on(method: string, pathPattern: string, handler: (req: TransportRequest) => unknown): this {
    this.handlers.set(`${method.toUpperCase()} ${pathPattern}`, handler);
    return this;
  }

  async fetch(req: TransportRequest): Promise<TransportResponse> {
    const cleanUrl = req.url.split("?")[0];
    const key = `${req.method.toUpperCase()} ${cleanUrl}`;
    const handler = this.handlers.get(key);

    if (!handler) {
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

    return {
      status: code !== 0 && code >= 400 ? code : 200,
      statusText: (envelope as { status: string }).status || "OK",
      headers: { "content-type": "application/json" },
      json: async () => envelope,
    };
  }
}

export interface RequestOptions<T = unknown> {
  pathParams?: Record<string, string | number>;
  query?: Record<
    string,
    string | number | boolean | readonly (string | number | boolean)[] | null | undefined
  >;
  headers?: Record<string, string>;
  signal?: AbortSignal;
  schema?: (data: unknown) => T;
}

export function buildUrl(
  path: string,
  pathParams?: Record<string, string | number>,
  query?: Record<
    string,
    string | number | boolean | readonly (string | number | boolean)[] | null | undefined
  >,
): string {
  let url = path;
  if (pathParams) {
    for (const [key, val] of Object.entries(pathParams)) {
      const encoded = encodeURIComponent(String(val));
      url = url.replaceAll(`:${key}`, encoded).replaceAll(`{${key}}`, encoded);
    }
  }
  if (query) {
    const searchParams = new URLSearchParams();
    for (const [k, v] of Object.entries(query)) {
      if (v === null || v === undefined) continue;
      if (Array.isArray(v)) {
        for (const item of v) {
          if (item !== null && item !== undefined) {
            searchParams.append(k, String(item));
          }
        }
      } else {
        searchParams.set(k, String(v));
      }
    }
    const qs = searchParams.toString();
    if (qs) {
      url += (url.includes("?") ? "&" : "?") + qs;
    }
  }
  return url;
}

let activeTransport: TransportAdapter = new FetchTransportAdapter();

async function executeRequestEnvelope<T>(
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
  path: string,
  body?: unknown,
  options: RequestOptions<T> = {},
): Promise<ApiResponse<T>> {
  const url = buildUrl(path, options.pathParams, options.query);
  const headers: Record<string, string> = {
    Accept: "application/json",
    ...options.headers,
  };

  let bodyString: string | undefined = undefined;
  if (body !== undefined) {
    bodyString = typeof body === "string" ? body : JSON.stringify(body);
    if (!headers["Content-Type"]) {
      headers["Content-Type"] = "application/json";
    }
  }

  let res: TransportResponse;
  try {
    res = await activeTransport.fetch({
      url,
      method,
      headers,
      body: bodyString,
      signal: options.signal,
    });
  } catch (err: unknown) {
    throw new ApiError(
      err instanceof Error ? err.message : "Network request failed",
      0,
      0,
      "NETWORK_ERROR",
      err,
    );
  }

  const json: unknown = await res.json().catch(() => null);

  const isObj = typeof json === "object" && json !== null;
  const rawCode = isObj && "code" in json ? Number((json as Record<string, unknown>).code) : 0;
  const rawStatus =
    isObj && "status" in json ? String((json as Record<string, unknown>).status) : "";
  const rawData = isObj && "data" in json ? (json as Record<string, unknown>).data : null;

  if (
    res.status >= 400 ||
    rawCode !== 0 ||
    (rawStatus && rawStatus !== "OK" && rawStatus !== "HEALTHY")
  ) {
    const errorDetails = typeof rawData === "string" ? rawData : "";
    const statusMsg = rawStatus || res.statusText || "ERROR";
    const httpStatus = res.status >= 400 ? res.status : rawCode >= 400 ? rawCode : 500;
    const finalMessage = errorDetails || `API Error (${httpStatus}): ${statusMsg}`;
    throw new ApiError(finalMessage, httpStatus, rawCode || httpStatus, statusMsg, rawData);
  }

  const payload =
    (rawData !== null && rawData !== undefined) || (isObj && "data" in json) ? rawData : json;

  let validatedData: T;
  if (options.schema) {
    try {
      validatedData = options.schema(payload);
    } catch (err) {
      if (err instanceof ApiError) throw err;
      throw new ContractViolationError(
        err instanceof Error ? err.message : "Schema validation failed",
        payload,
      );
    }
  } else {
    validatedData = payload as T;
  }

  const code = parseCode(rawCode);
  const status = rawStatus ? parseStatus(rawStatus) : Status.Ok;

  return {
    code,
    status,
    data: validatedData,
  };
}

async function executeRequest<T>(
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
  path: string,
  body?: unknown,
  options: RequestOptions<T> = {},
): Promise<T> {
  const envelope = await executeRequestEnvelope<T>(method, path, body, options);
  return envelope.data;
}

/**
 * Deep, high-leverage API client for CoSave.
 */
export const api = {
  get<T>(path: string, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("GET", path, undefined, options);
  },
  post<T>(path: string, body?: unknown, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("POST", path, body, options);
  },
  put<T>(path: string, body?: unknown, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("PUT", path, body, options);
  },
  patch<T>(path: string, body?: unknown, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("PATCH", path, body, options);
  },
  delete<T>(path: string, options?: RequestOptions<T>): Promise<T> {
    return executeRequest<T>("DELETE", path, undefined, options);
  },
  setTransport(adapter: TransportAdapter): () => void {
    const prev = activeTransport;
    activeTransport = adapter;
    return () => {
      activeTransport = prev;
    };
  },
  getTransport(): TransportAdapter {
    return activeTransport;
  },
};

/**
 * Backwards-compatibility wrapper around api for existing consumers.
 */
export async function apiFetch<T>(
  url: string,
  init: RequestInit = {},
  parser?: (data: unknown) => T,
): Promise<ApiResponse<T>> {
  const method = (init.method?.toUpperCase() ?? "GET") as
    "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  let body: unknown = undefined;
  if (init.body) {
    try {
      body = typeof init.body === "string" ? JSON.parse(init.body) : init.body;
    } catch {
      body = init.body;
    }
  }
  return executeRequestEnvelope<T>(method, url, body, {
    headers: init.headers as Record<string, string>,
    signal: init.signal ?? undefined,
    schema: parser,
  });
}

// Re-export feature types and API clients
export * from "./features/auth/types";
export * from "./features/categories/types";
export { authApi } from "./features/auth/api";
export { categoriesApi } from "./features/categories/api";
