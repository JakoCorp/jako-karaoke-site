import createClient from "openapi-fetch";

import type { paths } from "./generated";

/** Typed OpenAPI client bound to the app's backend spec. */
export const api = createClient<paths>({
  baseUrl: import.meta.env.VITE_API_URL ?? "",
  credentials: "include",
});

/** Returns the server's `{ error: string }` message from an error payload, if it has one. */
export function serverMessage(detail: unknown): string | undefined {
  if (typeof detail === "string" && detail !== "") return detail;
  if (typeof detail === "object" && detail !== null && "error" in detail) {
    const { error } = detail;
    if (typeof error === "string" && error !== "") return error;
  }
  return undefined;
}

/** Represents a failed API response with an HTTP status and detail payload. */
export class ApiError extends Error {
  readonly status: number;
  readonly detail: unknown;

  /** The message the server sent for this failure, safe to show to the user for 4xx statuses. */
  readonly serverMessage: string | undefined;

  constructor(status: number, detail: unknown) {
    const message = serverMessage(detail);
    super(message ?? `API error ${String(status)}`);
    this.name = "ApiError";
    this.status = status;
    this.detail = detail;
    this.serverMessage = message;
  }
}

/**
 * Returns the data of an `openapi-fetch` result and throws an {@link ApiError} when the
 * request failed. Lets mutations report failures through React Query's `onError`.
 */
export function unwrap<Result extends { data?: unknown; error?: unknown; response: Response }>(
  result: Result,
): NonNullable<Result["data"]> {
  if (result.data !== undefined && result.data !== null) return result.data;
  throw new ApiError(result.response.status, result.error);
}

/** Throws an {@link ApiError} when an `openapi-fetch` result for a no content request failed. */
export function ensureOk(result: { error?: unknown; response: Response }): void {
  if (result.error !== undefined) {
    throw new ApiError(result.response.status, result.error);
  }
}
