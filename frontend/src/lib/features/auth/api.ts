import { apiFetch, parseNull, type ApiResponse } from "$lib/api";
import {
  parseNullableUserDto,
  parseUserDto,
  type LoginPayload,
  type RegisterPayload,
  type UserDto,
} from "./types";

/**
 * Auth API service functions.
 */
export const authApi = {
  async register(payload: RegisterPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>(
      "/api/v1/auth/register",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseNullableUserDto,
    );
  },

  async login(payload: LoginPayload): Promise<ApiResponse<UserDto | null>> {
    return apiFetch<UserDto | null>(
      "/api/v1/auth/login",
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
      parseNullableUserDto,
    );
  },

  async logout(): Promise<ApiResponse<null>> {
    return apiFetch<null>(
      "/api/v1/auth/logout",
      {
        method: "POST",
      },
      parseNull,
    );
  },

  async me(): Promise<ApiResponse<UserDto>> {
    return apiFetch<UserDto>("/api/v1/auth/me", {}, parseUserDto);
  },
};
