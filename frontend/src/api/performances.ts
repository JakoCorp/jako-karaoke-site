import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

export type PerformanceSummary = components["schemas"]["PerformanceSummary"];
export type PerformanceResponse = components["schemas"]["PerformanceResponse"];
export type PerformanceTagKind = components["schemas"]["PerformanceTagKind"];

export const PERFORMANCE_TAG_KINDS = [
  "instrument",
  "modifier",
  "misc",
] as const satisfies readonly PerformanceTagKind[];

/** Query parameters accepted by the performances list endpoint. */
export type PerformanceListParams = SearchPaginationParams & {
  /** Field to sort by. Defaults to performance_date. */
  sort?: "performance_date" | "play_count" | "duration";
  /** Sort direction. Defaults to desc. */
  sort_dir?: "asc" | "desc";
};

/** Performance endpoints. */
export const performancesApi = {
  /** Returns a paginated, optionally filtered list of performances. */
  list: (params?: PerformanceListParams) =>
    api.GET("/api/performances", { params: { query: params } }),

  /** Returns a single performance by ID. */
  get: (id: string) => api.GET("/api/performances/{id}", { params: { path: { id } } }),

  /**
   * Returns lyrics for a performance.
   * Falls back to the linked song's lyrics if no performance-specific override is set.
   * Returns 404 if neither the performance nor the song has lyrics.
   */
  getLyrics: (id: string) => api.GET("/api/performances/{id}/lyrics", { params: { path: { id } } }),

  /** Creates a new performance. */
  create: (body: components["schemas"]["CreatePerformanceRequest"]) =>
    api.POST("/api/performances", { body }),

  /** Updates a performance by ID. */
  update: (id: string, body: components["schemas"]["UpdatePerformanceRequest"]) =>
    api.PUT("/api/performances/{id}", { params: { path: { id } }, body }),

  /** Deletes a performance by ID. */
  delete: (id: string) => api.DELETE("/api/performances/{id}", { params: { path: { id } } }),
};
