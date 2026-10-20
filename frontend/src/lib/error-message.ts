import { ApiError, serverMessage } from "@/api/client";

function failureReason(error: unknown, fallback: string): string {
  if (error instanceof ApiError) {
    const isServerFailure = error.status >= 500;
    return !isServerFailure && error.serverMessage ? error.serverMessage : fallback;
  }
  if (error instanceof Error && error.message !== "") return error.message;
  return serverMessage(error) ?? fallback;
}

/**
 * Returns a message that is safe to show to the user for a thrown or returned error.
 *
 * The server's messages start in lowercase, so the first letter is capitalized.
 */
export function errorMessage(error: unknown, fallback: string): string {
  const reason = failureReason(error, fallback);
  return reason.charAt(0).toUpperCase() + reason.slice(1);
}

/** Appends the reason for a failure to a summary, or returns the summary alone without one. */
export function failureMessage(summary: string, error: unknown): string {
  const reason = failureReason(error, "");
  return reason === "" ? summary : `${summary}: ${reason}`;
}
