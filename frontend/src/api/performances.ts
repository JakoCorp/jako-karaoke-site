import { api } from "./client";
import type { PaginationParams } from "./types";

/** Query parameters accepted by the performances list endpoint. */
export type PerformanceListParams = PaginationParams & {
  /** Text search across performance title, song title, and singer names. */
  q?: string;
  /** Field to sort by. Defaults to performance_date. */
  sort?: "performance_date" | "play_count" | "duration";
  /** Sort direction. Defaults to desc. */
  sort_dir?: "asc" | "desc";
};

/** Performance endpoints. */
export const performances = {
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
};
