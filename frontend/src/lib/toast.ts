import { toast } from "svelte-sonner";
import { ApiError } from "$lib/api/contracts";
import { InvariantViolationError } from "$lib/types";

export interface ToastErrorOptions {
  /** Optional custom title fallback */
  title?: string;
  /** Duration in milliseconds (defaults to 5000) */
  duration?: number;
}

/**
 * Extracts a user-friendly title, descriptive message, and action token from an unknown error.
 */
export function formatErrorMessage(err: unknown): {
  title: string;
  message: string;
  action?: string;
} {
  if (err instanceof ApiError) {
    const action = err.action ?? undefined;
    const title = err.apiStatus ? `${err.apiStatus.replace(/_/g, " ")}` : "Request Failed";
    // Capitalize title
    const formattedTitle = title
      .toLowerCase()
      .split(" ")
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");

    return {
      title: formattedTitle,
      message: err.message,
      action,
    };
  }

  if (err instanceof InvariantViolationError) {
    return {
      title: "State Invariant Violation",
      message: err.message,
      action: err.action,
    };
  }

  if (err instanceof Error) {
    return {
      title: "Unexpected Error",
      message: err.message,
    };
  }

  return {
    title: "Unexpected Error",
    message: typeof err === "string" ? err : "An unexpected failure occurred",
  };
}

/**
 * Dispatches a visible sonner toast for any error while preserving console error logging.
 *
 * Handles ApiError, InvariantViolationError, standard Error, and unknown thrown objects.
 */
export function errorToToast(err: unknown, options?: ToastErrorOptions): string | number {
  // Always keep authoritative console logging with full error context
  console.error("[cosave:error]", err);

  const { title: defaultTitle, message, action } = formatErrorMessage(err);
  const title = options?.title ?? defaultTitle;

  return toast.error(title, {
    description: action ? `${message} (${action})` : message,
    duration: options?.duration ?? 5000,
  });
}
