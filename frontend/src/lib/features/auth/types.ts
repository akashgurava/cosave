import { ContractViolationError, isObject } from "$lib/api";

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
  role: Role;
  created_at: number;
}

/**
 * Registration request payload.
 */
export interface RegisterPayload {
  name: string;
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
 * Validates and narrows an unknown value to a Role enum.
 */
export function parseRole(raw: unknown): Role {
  if (raw === "admin" || raw === "member") {
    return raw;
  }
  throw new ContractViolationError(`Invalid user role: ${JSON.stringify(raw)}`);
}

/**
 * Validates and narrows raw JSON data to a strongly-typed UserDto.
 */
export function parseUserDto(raw: unknown): UserDto {
  if (!isObject(raw)) {
    throw new ContractViolationError("UserDto payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("UserDto.id must be a string", raw);
  }
  if (typeof raw.name !== "string") {
    throw new ContractViolationError("UserDto.name must be a string", raw);
  }
  if (typeof raw.created_at !== "number") {
    throw new ContractViolationError("UserDto.created_at must be a number", raw);
  }
  return {
    id: raw.id,
    name: raw.name,
    role: parseRole(raw.role),
    created_at: raw.created_at,
  };
}

/**
 * Validates and narrows nullable raw JSON data to UserDto or null.
 */
export function parseNullableUserDto(raw: unknown): UserDto | null {
  if (raw === null || raw === undefined) {
    return null;
  }
  return parseUserDto(raw);
}
