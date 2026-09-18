/**
 * Standard API error codes matching the Rust backend.
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

/**
 * Standard status strings returned in API response envelopes.
 */
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

/**
 * Canonical JSON response envelope matching backend `ApiResponse<T>`.
 */
export interface ApiResponse<T> {
  code: Code;
  status: Status;
  data: T;
}

/**
 * Base error for API communication failures or contract mismatches.
 */
export class ApiError extends Error {
  public readonly details: unknown;
  public readonly httpStatus: number;
  public readonly apiStatus: Status | null;
  public readonly code: Code | null;

  constructor(
    message: string,
    details: unknown = null,
    httpStatus = 500,
    apiStatus: Status | null = null,
    code: Code | null = null,
  ) {
    super(message);
    this.name = "ApiError";
    this.details = details;
    this.httpStatus = httpStatus;
    this.apiStatus = apiStatus;
    this.code = code;
  }
}

/**
 * Thrown when an endpoint returns an unexpected numeric response code.
 */
export class UnanticipatedCodeError extends ApiError {
  constructor(public readonly rawCode: unknown) {
    super(`Unanticipated API response code: ${JSON.stringify(rawCode)}`);
    this.name = "UnanticipatedCodeError";
  }
}

/**
 * Thrown when an endpoint returns an unrecognized status string.
 */
export class UnanticipatedStatusError extends ApiError {
  constructor(public readonly rawStatus: unknown) {
    super(`Unanticipated API response status: ${JSON.stringify(rawStatus)}`);
    this.name = "UnanticipatedStatusError";
  }
}

/**
 * Thrown when an API response payload violates its expected schema contract.
 */
export class ContractViolationError extends ApiError {
  constructor(message: string, details: unknown = null) {
    super(`API Contract Violation: ${message}`, details);
    this.name = "ContractViolationError";
  }
}

/**
 * Type guard verifying whether a value is a non-null, non-array object record.
 */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Validates and narrows an unknown value to a known API Code.
 */
export function parseCode(rawCode: unknown): Code {
  switch (rawCode) {
    case 0:
    case Code.Zero:
      return Code.Zero;
    case 400:
    case Code.BadRequest:
      return Code.BadRequest;
    case 401:
    case Code.Unauthorized:
      return Code.Unauthorized;
    case 409:
    case Code.Conflict:
      return Code.Conflict;
    case 500:
    case Code.InternalError:
      return Code.InternalError;
    default:
      throw new UnanticipatedCodeError(rawCode);
  }
}

/**
 * Validates and narrows an unknown value to a known API Status.
 */
export function parseStatus(rawStatus: unknown): Status {
  switch (rawStatus) {
    case "HEALTHY":
    case Status.Healthy:
      return Status.Healthy;
    case "OK":
    case Status.Ok:
      return Status.Ok;
    case "BAD_REQUEST":
    case Status.BadRequest:
      return Status.BadRequest;
    case "UNAUTHENTICATED":
    case Status.Unauthenticated:
      return Status.Unauthenticated;
    case "INVALID_CREDENTIALS":
    case Status.InvalidCredentials:
      return Status.InvalidCredentials;
    case "USER_EXISTS":
    case Status.UserExists:
      return Status.UserExists;
    case "INTERNAL_ERROR":
    case Status.InternalError:
      return Status.InternalError;
    default:
      throw new UnanticipatedStatusError(rawStatus);
  }
}

/**
 * Validates and narrows raw JSON data to null.
 */
export function parseNull(raw: unknown): null {
  if (raw === null || raw === undefined) {
    return null;
  }
  throw new ContractViolationError(`Expected null response data, got: ${JSON.stringify(raw)}`);
}

/**
 * Extracts and validates the numeric code from an API response object.
 */
export function extractCode(response: unknown): Code {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  return parseCode((response as Record<string, unknown>).code);
}

/**
 * Extracts and validates the status string from an API response object.
 */
export function extractStatus(response: unknown): Status {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  return parseStatus((response as Record<string, unknown>).status);
}

/**
 * Extracts the payload data from an API response, verifying code and status first.
 */
export function extractData<T>(response: unknown, parser?: (data: unknown) => T): T {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  parseCode(res.code);
  parseStatus(res.status);
  return parser ? parser(res.data) : (res.data as T);
}

/**
 * Validates and extracts a typed ApiResponse envelope from an unknown response object.
 * When a parser is provided, response data is strictly validated and narrowed at runtime.
 */
export function extractApiResponse<T>(
  response: unknown,
  parser?: (data: unknown) => T,
): ApiResponse<T> {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  const code = parseCode(res.code);
  const status = parseStatus(res.status);
  const data = parser ? parser(res.data) : (res.data as T);
  return {
    code,
    status,
    data,
  };
}

/**
 * Typed wrapper around fetch that parses and validates the standardized API response envelope.
 */
export async function apiFetch<T>(
  url: string,
  init: RequestInit = {},
  parser?: (data: unknown) => T,
): Promise<ApiResponse<T>> {
  const headers = new Headers(init.headers || {});
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }

  const res = await fetch(url, {
    credentials: "same-origin",
    ...init,
    headers,
  });

  const json: unknown = await res.json().catch(() => null);

  if (!res.ok) {
    if (json && typeof json === "object") {
      try {
        const envelope = extractApiResponse<unknown>(json);
        const apiErr = new ApiError(
          `API Error: ${envelope.status}`,
          envelope.data,
          res.status,
          envelope.status,
          envelope.code,
        );
        console.warn(
          `[cosave:api] ${init.method || "GET"} ${url} failed (${res.status}):`,
          envelope.status,
        );
        throw apiErr;
      } catch (err) {
        if (err instanceof ApiError) {
          throw err;
        }
      }
    }
    const httpErr = new ApiError(`HTTP ${res.status}: ${res.statusText}`, null, res.status);
    console.warn(`[cosave:api] ${init.method || "GET"} ${url} HTTP failure:`, httpErr.message);
    throw httpErr;
  }

  try {
    return extractApiResponse<T>(json, parser);
  } catch (err) {
    console.error(`[cosave:api] ${init.method || "GET"} ${url} contract violation:`, err);
    throw err;
  }
}

// Re-export feature types and API clients for backwards compatibility
export * from "./features/auth/types";
export * from "./features/categories/types";
export { authApi } from "./features/auth/api";
export { categoriesApi } from "./features/categories/api";
