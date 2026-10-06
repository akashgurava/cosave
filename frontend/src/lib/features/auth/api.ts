/**
 * Auth API service functions with runtime schema contract enforcement.
 *
 * Dispatches requests to backend endpoints under `/api/v1/auth`, enforcing
 * strict schema validation on inbound user envelopes.
 */

import { api } from "$lib/api";
import { parseNull } from "$lib/api/contracts";
import { parseUserDto, type LoginPayload, type RegisterPayload, type UserDto } from "./types";

export const authApi = {
  register(payload: RegisterPayload): Promise<UserDto> {
    return api.post<UserDto>("/api/v1/auth/register", payload, {
      schema: parseUserDto,
    });
  },

  login(payload: LoginPayload): Promise<UserDto> {
    return api.post<UserDto>("/api/v1/auth/login", payload, {
      schema: parseUserDto,
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
