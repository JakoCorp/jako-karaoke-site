import { api } from "./client";
import type { components } from "./generated";

export type PlaylistResponse = components["schemas"]["PlaylistResponse"];
export type PlaylistEntry = components["schemas"]["PlaylistEntry"];
export type PlaylistKind = components["schemas"]["PlaylistKind"];
export type CreatePlaylistRequest = components["schemas"]["CreatePlaylistRequest"];

export type PlaylistListParams = {
  page?: number;
  per_page?: number;
  q?: string;
};

/** Playlist endpoints. */
export const playlists = {
  /** Returns a paginated, optionally filtered list of public playlists. */
  list: (params?: PlaylistListParams) => api.GET("/api/playlists", { params: { query: params } }),

  /** Returns a single playlist by ID. */
  get: (id: string) => api.GET("/api/playlists/{id}", { params: { path: { id } } }),

  /** Returns the ordered performances in a playlist. */
  getPerformances: (id: string) =>
    api.GET("/api/playlists/{id}/performances", { params: { path: { id } } }),

  /** Returns a paginated, optionally filtered list of playlists for a user. */
  listByUser: (userId: string, params?: PlaylistListParams) =>
    api.GET("/api/users/{id}/playlists", { params: { path: { id: userId }, query: params } }),

  /** Returns the ordered entries in a user's favorites playlist. */
  getFavorites: (userId: string) =>
    api.GET("/api/users/{id}/favorites", { params: { path: { id: userId } } }),

  /** Creates a new playlist. */
  create: (body: CreatePlaylistRequest) => api.POST("/api/playlists", { body }),

  /** Appends performances to a playlist. */
  addPerformances: (id: string, performanceIds: string[]) =>
    api.POST("/api/playlists/{id}/performances", {
      params: { path: { id } },
      body: { performance_ids: performanceIds },
    }),

  /** Removes performances from a playlist. */
  removePerformances: (id: string, performanceIds: string[]) =>
    api.DELETE("/api/playlists/{id}/performances", {
      params: { path: { id } },
      body: { performance_ids: performanceIds },
    }),

  /** Returns IDs of a user's playlists that contain the given performance. */
  getPlaylistsContaining: (userId: string, performanceId: string) =>
    api.GET("/api/users/{id}/playlists/containing/{performance_id}", {
      params: { path: { id: userId, performance_id: performanceId } },
    }),

  /** Deletes a playlist by ID. */
  delete: (id: string) => api.DELETE("/api/playlists/{id}", { params: { path: { id } } }),
};
