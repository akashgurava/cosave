import { describe, it, expect } from "vitest";
import { Role, parseRole, parseUserDto } from "./types";
import { ContractViolationError } from "$lib/api/contracts";

describe("Auth Types & Schema Decoders", () => {
  describe("parseRole", () => {
    it("parses valid roles", () => {
      expect(parseRole("admin")).toBe(Role.Admin);
      expect(parseRole("member")).toBe(Role.Member);
    });

    it("throws ContractViolationError on invalid role", () => {
      expect(() => parseRole("superuser")).toThrow(ContractViolationError);
      expect(() => parseRole(null)).toThrow(ContractViolationError);
      expect(() => parseRole(123)).toThrow(ContractViolationError);
    });
  });

  describe("parseUserDto", () => {
    it("parses valid UserDto payload", () => {
      const raw = {
        id: "usr_abc",
        username: "dana",
        role: "admin",
        createdAt: 1700000000,
      };
      const user = parseUserDto(raw);
      expect(user.id).toBe("usr_abc");
      expect(user.username).toBe("dana");
      expect(user.role).toBe("admin");
      expect(user.createdAt).toBe(1700000000);
      expect(Object.isFrozen(user)).toBe(true);
    });

    it("throws ContractViolationError on missing or invalid fields", () => {
      expect(() => parseUserDto(null)).toThrow(ContractViolationError);
      expect(() => parseUserDto([])).toThrow(ContractViolationError);
      expect(() =>
        parseUserDto({ id: 123, username: "dana", role: "admin", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseUserDto({ id: "u1", username: 456, role: "admin", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseUserDto({ id: "u1", username: "dana", role: "guest", createdAt: 100 }),
      ).toThrow(ContractViolationError);
      expect(() =>
        parseUserDto({ id: "u1", username: "dana", role: "admin", createdAt: "recent" }),
      ).toThrow(ContractViolationError);
    });
  });
});
