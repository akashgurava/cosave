/**
 * Wire envelopes, response codes, and runtime contract errors.
 *
 * Defines the standard shape of responses returned by the Rust Axum backend,
 * the unified ApiError class, and runtime schema validation primitives.
 */

export const Code = {
  Zero: 0,
  BadRequest: 400,
  Unauthorized: 401,
  Conflict: 409,
  InternalError: 500,
} as const;

export type Code = (typeof Code)[keyof typeof Code];

export const Status = {
  Healthy: "HEALTHY",
  Ok: "OK",
  BadRequest: "BAD_REQUEST",
  Unauthenticated: "UNAUTHENTICATED",
  InvalidCredentials: "INVALID_CREDENTIALS",
  UserAlreadyExists: "USER_ALREADY_EXISTS",
  InternalError: "INTERNAL_ERROR",
} as const;

export type Status = (typeof Status)[keyof typeof Status];

export interface ApiResponse<T> {
  code: Code;
  status: Status;
  data: T;
}

export interface ErrorPayload {
  action: string;
  message: string;
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
    public readonly action: string | null = null,
  ) {
    super(message);
  }
}

export function parseCode(rawCode: unknown): Code {
  if (rawCode === Code.Zero || rawCode === 0) return Code.Zero;
  if (rawCode === Code.BadRequest || rawCode === 400) return Code.BadRequest;
  if (rawCode === Code.Unauthorized || rawCode === 401) return Code.Unauthorized;
  if (rawCode === Code.Conflict || rawCode === 409) return Code.Conflict;
  if (rawCode === Code.InternalError || rawCode === 500) return Code.InternalError;
  throw new ApiError(
    `Unanticipated API response code: ${JSON.stringify(rawCode)}`,
    500,
    500,
    "INTERNAL_ERROR",
  );
}

export function parseStatus(rawStatus: unknown): Status {
  if (rawStatus === Status.Healthy || rawStatus === "HEALTHY") return Status.Healthy;
  if (rawStatus === Status.Ok || rawStatus === "OK") return Status.Ok;
  if (rawStatus === Status.BadRequest || rawStatus === "BAD_REQUEST") return Status.BadRequest;
  if (rawStatus === Status.Unauthenticated || rawStatus === "UNAUTHENTICATED")
    return Status.Unauthenticated;
  if (rawStatus === Status.InvalidCredentials || rawStatus === "INVALID_CREDENTIALS")
    return Status.InvalidCredentials;
  if (rawStatus === Status.UserAlreadyExists || rawStatus === "USER_ALREADY_EXISTS")
    return Status.UserAlreadyExists;
  if (rawStatus === Status.InternalError || rawStatus === "INTERNAL_ERROR")
    return Status.InternalError;
  throw new ApiError(
    `Unanticipated API response status: ${JSON.stringify(rawStatus)}`,
    500,
    500,
    "INTERNAL_ERROR",
  );
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
