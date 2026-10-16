import { toast } from "svelte-sonner";
import { ApiError } from "$lib/api/contracts";
import {
  IdCreationError,
  InvariantViolationError,
  ParseEnumError,
  ParseModelError,
} from "$lib/errors";

export { toast };

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
  action: string | null;
} {
  if (err instanceof ApiError) {
    const action = err.action;
    const title =
      err.apiStatus !== null && err.apiStatus !== undefined && err.apiStatus !== ""
        ? `${err.apiStatus.replace(/_/g, " ")}`
        : "Request Failed";
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

  if (err instanceof IdCreationError) {
    return {
      title: "ID Creation Error",
      message: err.message,
      action: err.action,
    };
  }

  if (err instanceof ParseModelError) {
    return {
      title: "Validation Error",
      message: err.message,
      action: null,
    };
  }

  if (err instanceof ParseEnumError) {
    return {
      title: "Invalid Enum Value",
      message: err.message,
      action: null,
    };
  }

  if (err instanceof Error) {
    return {
      title: "Unexpected Error",
      message: err.message,
      action: null,
    };
  }

  return {
    title: "Unexpected Error",
    message: typeof err === "string" ? err : "An unexpected failure occurred",
    action: null,
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
  const title = options !== undefined && options.title !== undefined ? options.title : defaultTitle;
  const duration =
    options !== undefined && options.duration !== undefined ? options.duration : 5000;
  const description = action !== null && action !== "" ? `${message} (${action})` : message;

  return toast.error(title, {
    description,
    duration,
  });
}
