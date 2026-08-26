import createClient from "openapi-fetch";

import type { paths } from "./generated";

/** Typed OpenAPI client bound to the app's backend spec. */
export const api = createClient<paths>({ baseUrl: "" });

/** Represents a failed API response with an HTTP status and detail payload. */
export class ApiError extends Error {
  readonly status: number;
  readonly detail: unknown;

  constructor(status: number, detail: unknown) {
    super(`API error ${String(status)}`);
    this.name = "ApiError";
    this.status = status;
    this.detail = detail;
  }
}
