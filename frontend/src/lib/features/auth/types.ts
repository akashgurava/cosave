/**
 * Authentication domain entities, DTOs, and runtime schema decoders.
 *
 * Implements "Parse, Don't Validate" decoders for UserDto and Role enums,
 * returning immutable, verified models across network boundaries.
 */

import { ContractViolationError, isObject } from "$lib/api/contracts";

/**
 * System roles available for user accounts.
 */
export type Role = "admin" | "member";

/**
 * Public user representation returned by auth endpoints.
 */
export interface UserDto {
  readonly id: string;
  readonly username: string;
  readonly role: Role;
  readonly createdAt: number;
}

/**
 * Registration request payload.
 */
export interface RegisterPayload {
  readonly username: string;
  readonly password: string;
}

/**
 * Login request payload.
 */
export interface LoginPayload {
  readonly username: string;
  readonly password: string;
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
  if (isObject(raw) === false) {
    throw new ContractViolationError("UserDto payload must be an object", raw);
  }
  if (typeof raw.id !== "string") {
    throw new ContractViolationError("UserDto.id must be a string", raw);
  }
  if (typeof raw.username !== "string") {
    throw new ContractViolationError("UserDto.username must be a string", raw);
  }
  if (typeof raw.createdAt !== "number") {
    throw new ContractViolationError("UserDto.createdAt must be a number", raw);
  }
  return Object.freeze({
    id: raw.id,
    username: raw.username,
    role: parseRole(raw.role),
    createdAt: raw.createdAt,
  });
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
