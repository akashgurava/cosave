import { api, parseNull } from "$lib/api";
import {
  parseNullableUserDto,
  parseUserDto,
  type LoginPayload,
  type RegisterPayload,
  type UserDto,
} from "./types";

/**
 * Auth API service functions with runtime schema contract enforcement.
 */
export const authApi = {
  register(payload: RegisterPayload): Promise<UserDto | null> {
    return api.post<UserDto | null>("/api/v1/auth/register", payload, {
      schema: parseNullableUserDto,
    });
  },

  login(payload: LoginPayload): Promise<UserDto | null> {
    return api.post<UserDto | null>("/api/v1/auth/login", payload, {
      schema: parseNullableUserDto,
    });
  },

  logout(): Promise<null> {
    return api.post<null>("/api/v1/auth/logout", undefined, {
      schema: parseNull,
    });
  },

  me(): Promise<UserDto> {
    return api.get<UserDto>("/api/v1/auth/me", {
      schema: parseUserDto,
    });
  },
};
