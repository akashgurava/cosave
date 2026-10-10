import { describe, it, expect } from "vitest";
import { render } from "svelte/server";
import AuthModal from "./AuthModal.svelte";

describe("AuthModal Component UI (Tier 1 / Presentation)", () => {
  it("renders embedded in-place Auth Wall correctly with card header and form", () => {
    const { body } = render(AuthModal, {
      props: {
        embedded: true,
        isOpen: true,
      },
    });

    // 1. Authoritative header for embedded card
    expect(body).toContain("Sign In to CoSave");
    expect(body).toContain("Access your family finance dashboard and accounts.");

    // 2. Form fields with unadorned IDs
    expect(body).toContain('id="auth-username"');
    expect(body).toContain('id="auth-password"');

    // 3. Submit action button
    expect(body).toContain("Sign In");

    // 4. Mode switcher tabs
    expect(body).toContain("Register");

    // 5. Embedded return to landing page navigation link
    expect(body).toContain("← Back to Home");

    // 6. Embedded mode does NOT render modal overlay backdrop button
    expect(body).not.toContain('aria-label="Close dialog overlay"');
  });

  it("renders modal dialog overlay and close button when not embedded", () => {
    const { body } = render(AuthModal, {
      props: {
        embedded: false,
        isOpen: true,
        onClose: () => {},
      },
    });

    expect(body).toContain('role="dialog"');
    expect(body).toContain('aria-label="Close dialog overlay"');
    expect(body).toContain('aria-label="Close dialog"');
    // Embedded link is not shown in floating modal
    expect(body).not.toContain("&larr; Back to Home");
  });
});
