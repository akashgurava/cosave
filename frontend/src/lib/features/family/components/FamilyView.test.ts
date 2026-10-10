import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render } from "svelte/server";
import { api, ApiError } from "$lib/api";
import { MemoryTransportAdapter } from "$lib/api/testing";
import { familyStore } from "$lib/features/family/store.svelte";
import FamilyView from "./FamilyView.svelte";
import { toast } from "svelte-sonner";
import { toCurrencyId, toFamilyId, toMemberId, type FamilyDetails } from "../types";

describe("Family Feature Failure Cases & UI Error Presentation", () => {
  let memoryTransport: MemoryTransportAdapter;
  let restoreTransport: (() => void) | undefined;

  beforeEach(() => {
    vi.restoreAllMocks();
    memoryTransport = new MemoryTransportAdapter();
    restoreTransport = api.setTransport(memoryTransport);
    familyStore.reset();
  });

  afterEach(() => {
    if (restoreTransport !== undefined) {
      restoreTransport();
    }
    familyStore.reset();
  });

  describe("FamilyView Error Presentation (Tier 1 / Presentation)", () => {
    it("renders actionable error alert with Retry button when store fails to load", async () => {
      const toastSpy = vi.spyOn(toast, "error").mockImplementation(() => "toast-id");

      // Simulate network / server failure on family load
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: 500,
        status: "INTERNAL_ERROR",
        data: {
          action: "FAMILY.GET_DETAILS.DB_FAILURE",
          message: "Database connection failed",
        },
      }));

      await familyStore.load();
      expect(familyStore.error).toBe("Database connection failed");
      expect(toastSpy).toHaveBeenCalledWith(
        "Internal Error",
        expect.objectContaining({
          description: "Database connection failed (FAMILY.GET_DETAILS.DB_FAILURE)",
        }),
      );

      const { body } = render(FamilyView);
      expect(body).toContain('role="alert"');
      expect(body).toContain("Database connection failed");
      expect(body).toContain("Retry");
    });
  });

  describe("Store Duplicate Member Toasting & Action Pinpointing", () => {
    it("triggers formatted toast with screaming action code when member already exists", async () => {
      const toastSpy = vi.spyOn(toast, "error").mockImplementation(() => "toast-id");

      // Seed family
      const initialDetails: FamilyDetails = {
        family: {
          id: toFamilyId(1),
          familyName: "Smiths",
          currencyId: toCurrencyId(1),
          createdAt: 100,
        },
        members: [
          { id: toMemberId(1), familyId: toFamilyId(1), memberName: "Sarah", createdAt: 100 },
        ],
        accounts: [],
        currencies: [
          {
            id: toCurrencyId(1),
            code: "USD",
            name: "US Dollar",
            symbol: "$",
            scale: 2,
            sortOrder: 1,
          },
        ],
      };
      memoryTransport.on("GET", "/api/v1/config/family", () => ({
        code: 0,
        status: "OK",
        data: initialDetails,
      }));
      await familyStore.load();

      // Attempt to add duplicate member "Sarah"
      memoryTransport.on("POST", "/api/v1/config/members", () => ({
        code: 409,
        status: "MEMBER_ALREADY_EXISTS",
        data: {
          action: "FAMILY.CREATE_MEMBER.ALREADY_EXISTS",
          message: "Member 'Sarah' already exists in this family.",
        },
      }));

      await expect(familyStore.addMember("Sarah")).rejects.toThrow(ApiError);

      expect(toastSpy).toHaveBeenCalledWith(
        "Member Already Exists",
        expect.objectContaining({
          description:
            "Member 'Sarah' already exists in this family. (FAMILY.CREATE_MEMBER.ALREADY_EXISTS)",
        }),
      );
    });
  });
});
