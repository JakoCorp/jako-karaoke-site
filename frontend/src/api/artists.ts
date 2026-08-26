import { api } from "./client";
import type { PaginationParams } from "./types";

/** Artist endpoints. */
export const artists = {
  /** Returns a paginated list of artists. */
  list: (params?: PaginationParams) => api.GET("/api/artists", { params: { query: params } }),

  /** Returns a single artist by ID. */
  get: (id: string) => api.GET("/api/artists/{id}", { params: { path: { id } } }),
};
