import { ApiError, serverMessage } from "@/api/client";

/** Returns a message that is safe to show to the user for a thrown or returned error. */
export function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof ApiError) {
    const isServerFailure = error.status >= 500;
    return !isServerFailure && error.serverMessage ? error.serverMessage : fallback;
  }
  if (error instanceof Error && error.message !== "") return error.message;
  return serverMessage(error) ?? fallback;
}

/** Appends the reason for a failure to a summary, or returns the summary alone without one. */
export function failureMessage(summary: string, error: unknown): string {
  const reason = errorMessage(error, "");
  return reason === "" ? summary : `${summary}: ${reason}`;
}
