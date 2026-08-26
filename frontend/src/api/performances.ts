import { api } from "./client";
import type { PaginationParams } from "./types";

/** Performance endpoints. */
export const performances = {
  /** Returns a paginated list of performances. */
  list: (params?: PaginationParams) => api.GET("/api/performances", { params: { query: params } }),

  /** Returns a single performance by ID. */
  get: (id: string) => api.GET("/api/performances/{id}", { params: { path: { id } } }),

  /**
   * Returns lyrics for a performance.
   * Falls back to the linked song's lyrics if no performance-specific override is set.
   * Returns 404 if neither the performance nor the song has lyrics.
   */
  getLyrics: (id: string) => api.GET("/api/performances/{id}/lyrics", { params: { path: { id } } }),
};
