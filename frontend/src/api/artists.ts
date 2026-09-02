import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

export type ArtistSummary = components["schemas"]["ArtistSummary"];
export type ArtistResponse = components["schemas"]["ArtistResponse"];
export type ArtistLinkInfo = components["schemas"]["ArtistLinkInfo"];
export type ArtistImageKind = components["schemas"]["ArtistImageKind"];

export const ARTIST_LINK_KINDS = [
  "youtube",
  "website",
  "x",
  "instagram",
  "twitch",
  "other",
] as const;
export type ArtistLinkKind = (typeof ARTIST_LINK_KINDS)[number];

/** Artist endpoints. */
export const artistsApi = {
  /** Returns a paginated list of artists. */
  list: (params?: SearchPaginationParams) => api.GET("/api/artists", { params: { query: params } }),

  /** Returns a single artist by ID. */
  get: (id: string) => api.GET("/api/artists/{id}", { params: { path: { id } } }),

  /** Creates a new artist. */
  create: (body: components["schemas"]["CreateArtistRequest"]) =>
    api.POST("/api/artists", { body }),

  /** Replaces all fields of an artist. */
  update: (id: string, body: components["schemas"]["UpdateArtistRequest"]) =>
    api.PUT("/api/artists/{id}", { params: { path: { id } }, body }),

  /** Deletes an artist by ID. */
  delete: (id: string) => api.DELETE("/api/artists/{id}", { params: { path: { id } } }),
};
