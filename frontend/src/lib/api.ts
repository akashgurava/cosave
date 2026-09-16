/**
 * Standard API error codes matching the Rust backend.
 */
export const Code = {
  Zero: 0,
  zero: (): 0 => 0,
} as const;

export type Code = (typeof Code)["Zero"];

/**
 * Standard status strings returned in API response envelopes.
 */
export const Status = {
  Healthy: "HEALTHY",
  Ok: "OK",
  healthy: (): "HEALTHY" => "HEALTHY",
  ok: (): "OK" => "OK",
} as const;

export type Status = (typeof Status)["Healthy" | "Ok"];

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

  constructor(message: string, details: unknown = null) {
    super(message);
    this.name = "ApiError";
    this.details = details;
  }
}

/**
 * Thrown when an endpoint returns an unexpected numeric response code.
 */
export class UnanticipatedCodeError extends ApiError {
  constructor(public readonly code: unknown) {
    super(`Unanticipated API response code: ${JSON.stringify(code)}`);
    this.name = "UnanticipatedCodeError";
  }
}

/**
 * Thrown when an endpoint returns an unrecognized status string.
 */
export class UnanticipatedStatusError extends ApiError {
  constructor(public readonly status: unknown) {
    super(`Unanticipated API response status: ${JSON.stringify(status)}`);
    this.name = "UnanticipatedStatusError";
  }
}

/**
 * Validates and narrows an unknown value to a known API Code.
 */
export function parseCode(rawCode: unknown): Code {
  if (rawCode === Code.Zero || rawCode === 0) {
    return Code.Zero;
  }
  throw new UnanticipatedCodeError(rawCode);
}

/**
 * Validates and narrows an unknown value to a known API Status.
 */
export function parseStatus(rawStatus: unknown): Status {
  if (rawStatus === Status.Healthy || rawStatus === "HEALTHY") {
    return Status.Healthy;
  }
  if (rawStatus === Status.Ok || rawStatus === "OK") {
    return Status.Ok;
  }
  throw new UnanticipatedStatusError(rawStatus);
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
export function extractData<T>(response: unknown): T {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  parseCode(res.code);
  parseStatus(res.status);
  return res.data as T;
}

/**
 * Validates and extracts a typed ApiResponse envelope from an unknown response object.
 */
export function extractApiResponse<T>(response: unknown): ApiResponse<T> {
  if (!response || typeof response !== "object") {
    throw new ApiError("Response is not an object", response);
  }
  const res = response as Record<string, unknown>;
  const code = parseCode(res.code);
  const status = parseStatus(res.status);
  return {
    code,
    status,
    data: res.data as T,
  };
}

/**
 * Typed wrapper around fetch that parses the standardized API response envelope.
 */
export async function apiFetch<T>(url: string, init: RequestInit = {}): Promise<ApiResponse<T>> {
  const res = await fetch(url, init);
  if (!res.ok) {
    throw new ApiError(`HTTP ${res.status}: ${res.statusText}`, { status: res.status });
  }
  const json: unknown = await res.json();
  return extractApiResponse<T>(json);
}
