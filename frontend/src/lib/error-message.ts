import { ApiError, serverMessage } from "@/api/client";

/**
 * Returns a message that is safe to show to the user for a thrown or returned error.
 *
 * Server messages are used for client errors only. Server failures and unrecognized values
 * fall back to the given message so internal details never reach the UI.
 */
export function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof ApiError) {
    const isServerFailure = error.status >= 500;
    return !isServerFailure && error.serverMessage ? error.serverMessage : fallback;
  }
  if (error instanceof Error && error.message !== "") return error.message;
  return serverMessage(error) ?? fallback;
}
