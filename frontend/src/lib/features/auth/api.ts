import { api } from "$lib/api";
import type { LoginPayload, RegisterPayload, UserDto } from "./types";

/**
 * Auth API service functions.
 */
export const authApi = {
  register(payload: RegisterPayload): Promise<UserDto | null> {
    return api.post<UserDto | null>("/api/v1/auth/register", payload);
  },

  login(payload: LoginPayload): Promise<UserDto | null> {
    return api.post<UserDto | null>("/api/v1/auth/login", payload);
  },

  logout(): Promise<null> {
    return api.post<null>("/api/v1/auth/logout");
  },

  me(): Promise<UserDto> {
    return api.get<UserDto>("/api/v1/auth/me");
  },
};
