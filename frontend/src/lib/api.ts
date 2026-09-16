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
 * System roles available for user accounts.
 */
export type Role = "admin" | "member";

/**
 * Public user representation returned by auth endpoints.
 */
export interface UserDto {
  id: string;
  name: string;
  email: string | null;
  role: Role;
  created_at: number;
}

/**
 * Registration request payload.
 */
export interface RegisterPayload {
  name: string;
  email?: string | null;
  password: string;
}

/**
 * Login request payload.
 */
export interface LoginPayload {
  name: string;
  password: string;
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
        throw new ApiError(
          `API Error: ${envelope.status}`,
          envelope.data,
          res.status,
          envelope.status,
          envelope.code,
        );
      } catch (err) {
        if (err instanceof ApiError) {
          throw err;
        }
      }
    }
    throw new ApiError(`HTTP ${res.status}: ${res.statusText}`, null, res.status);
  }

  return extractApiResponse<T>(json);
}

/**
 * Auth API service functions.
 */
export const authApi = {
  async register(payload: RegisterPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>("/api/v1/auth/register", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async login(payload: LoginPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>("/api/v1/auth/login", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  async logout(): Promise<ApiResponse<null>> {
    return apiFetch<null>("/api/v1/auth/logout", {
      method: "POST",
    });
  },

  async me(): Promise<ApiResponse<UserDto>> {
    return apiFetch<UserDto>("/api/v1/auth/me");
  },
};
