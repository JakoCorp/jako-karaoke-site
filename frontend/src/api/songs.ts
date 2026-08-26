import { api } from "./client";
import type { PaginationParams } from "./types";

/** Song endpoints. */
export const songs = {
  /** Returns a paginated list of songs. */
  list: (params?: PaginationParams) => api.GET("/api/songs", { params: { query: params } }),

  /** Returns a single song by ID. */
  get: (id: string) => api.GET("/api/songs/{id}", { params: { path: { id } } }),

  /** Returns the lyrics for a song, or 404 if none are set. */
  getLyrics: (id: string) => api.GET("/api/songs/{id}/lyrics", { params: { path: { id } } }),
};
