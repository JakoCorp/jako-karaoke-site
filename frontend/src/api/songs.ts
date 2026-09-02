import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

export type SongSummary = components["schemas"]["SongSummary"];
export type SongResponse = components["schemas"]["SongResponse"];
export type SongTagKind = components["schemas"]["SongTagKind"];

export const SONG_TAG_KINDS = [
  "genre",
  "source",
  "language",
  "misc",
] as const satisfies readonly SongTagKind[];

/** Song endpoints. */
export const songsApi = {
  /** Returns a paginated list of songs. */
  list: (params?: SearchPaginationParams) => api.GET("/api/songs", { params: { query: params } }),

  /** Returns a single song by ID. */
  get: (id: string) => api.GET("/api/songs/{id}", { params: { path: { id } } }),

  /** Returns the lyrics for a song, or 404 if none are set. */
  getLyrics: (id: string) => api.GET("/api/songs/{id}/lyrics", { params: { path: { id } } }),

  /** Creates a new song. */
  create: (body: components["schemas"]["CreateSongRequest"]) => api.POST("/api/songs", { body }),

  /** Updates a song by ID. */
  update: (id: string, body: components["schemas"]["UpdateSongRequest"]) =>
    api.PUT("/api/songs/{id}", { params: { path: { id } }, body }),

  /** Deletes a song by ID. */
  delete: (id: string) => api.DELETE("/api/songs/{id}", { params: { path: { id } } }),
};
