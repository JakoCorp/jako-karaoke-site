import { api } from "./client";
import type { components } from "./generated";
import type { PaginationParams } from "./types";

export type ArtistResponse = components["schemas"]["ArtistResponse"];

/** Artist endpoints. */
export const artists = {
  /** Returns a paginated list of artists. */
  list: (params?: PaginationParams) => api.GET("/api/artists", { params: { query: params } }),

  /** Returns a single artist by ID. */
  get: (id: string) => api.GET("/api/artists/{id}", { params: { path: { id } } }),
};
