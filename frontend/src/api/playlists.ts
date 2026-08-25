import { api } from "./client";

/** Playlist endpoints. */
export const playlists = {
  /** Returns all publicly visible playlists. */
  list: () => api.GET("/api/playlists", {}),

  /** Returns a single playlist by ID. */
  get: (id: string) => api.GET("/api/playlists/{id}", { params: { path: { id } } }),

  /** Returns the ordered performances in a playlist. */
  getPerformances: (id: string) =>
    api.GET("/api/playlists/{id}/performances", { params: { path: { id } } }),
};
