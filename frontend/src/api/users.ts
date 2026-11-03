import { api } from "./client";
import type { components } from "./generated";
import type { QueryOf } from "./types";

export type UserSummary = components["schemas"]["UserSummary"];
export type UserPerformancePlayEntry = components["schemas"]["UserPerformancePlayEntry"];
export type UserSoundbitePlayEntry = components["schemas"]["UserSoundbitePlayEntry"];

/** Query parameters of the user search endpoint. */
export type UserListParams = QueryOf<"search_users">;

/** Query parameters of the favorite soundbites list endpoint. */
export type FavoriteSoundbiteListParams = QueryOf<"list_user_favorite_soundbites">;

/** User endpoints. */
export const usersApi = {
  /** Returns a paginated list of users matching an optional username substring or id. */
  search: (params?: UserListParams) => api.GET("/api/users", { params: { query: params } }),

  /** Changes a user's username, bypassing the cooldown. Requires `capabilities:manage`. */
  updateUsername: (userId: string, username: string) =>
    api.PATCH("/api/users/{id}", {
      params: { path: { id: userId } },
      body: { username } satisfies components["schemas"]["UpdateUserRequest"],
    }),

  /** Returns capabilities assigned to a user. */
  listCapabilities: (userId: string) =>
    api.GET("/api/users/{id}/capabilities", { params: { path: { id: userId } } }),

  /** Grants a capability to a user. */
  grantCapability: (userId: string, capability: string) =>
    api.POST("/api/users/{id}/capabilities", {
      params: { path: { id: userId } },
      body: { capability },
    }),

  /** Revokes a capability from a user. */
  revokeCapability: (userId: string, capability: string) =>
    api.DELETE("/api/users/{id}/capabilities/{capability}", {
      params: { path: { id: userId, capability } },
    }),

  /** Returns a paginated list of the performances a user has played. */
  listPerformancePlays: (userId: string, page: number, perPage: number) =>
    api.GET("/api/users/{id}/plays/performances", {
      params: { path: { id: userId }, query: { page, per_page: perPage } },
    }),

  /** Removes every counted performance play of a user. */
  deleteAllPerformancePlays: (userId: string) =>
    api.DELETE("/api/users/{id}/plays/performances", { params: { path: { id: userId } } }),

  /** Removes a user's counted plays on one performance. */
  deletePlaysOnPerformance: (userId: string, performanceId: string) =>
    api.DELETE("/api/users/{id}/plays/performances/{performance_id}", {
      params: { path: { id: userId, performance_id: performanceId } },
    }),

  /** Returns a paginated list of the soundbites a user has played. */
  listSoundbitePlays: (userId: string, page: number, perPage: number) =>
    api.GET("/api/users/{id}/plays/soundbites", {
      params: { path: { id: userId }, query: { page, per_page: perPage } },
    }),

  /** Removes every counted soundbite play of a user. */
  deleteAllSoundbitePlays: (userId: string) =>
    api.DELETE("/api/users/{id}/plays/soundbites", { params: { path: { id: userId } } }),

  /** Removes a user's counted plays on one soundbite. */
  deletePlaysOnSoundbite: (userId: string, soundbiteId: string) =>
    api.DELETE("/api/users/{id}/plays/soundbites/{soundbite_id}", {
      params: { path: { id: userId, soundbite_id: soundbiteId } },
    }),

  /** Returns a paginated, optionally searched list of the soundbites a user has marked as a favorite. */
  listFavoriteSoundbites: (userId: string, params?: FavoriteSoundbiteListParams) =>
    api.GET("/api/users/{id}/favorites/soundbites", {
      params: { path: { id: userId }, query: params },
    }),

  /** Returns the IDs of every soundbite a user has marked as a favorite. */
  listFavoriteSoundbiteIds: (userId: string) =>
    api.GET("/api/users/{id}/favorites/soundbites/ids", { params: { path: { id: userId } } }),

  /** Marks a soundbite as a favorite of a user. */
  addFavoriteSoundbite: (userId: string, soundbiteId: string) =>
    api.PUT("/api/users/{id}/favorites/soundbites/{soundbite_id}", {
      params: { path: { id: userId, soundbite_id: soundbiteId } },
    }),

  /** Removes a soundbite from the favorites of a user. */
  removeFavoriteSoundbite: (userId: string, soundbiteId: string) =>
    api.DELETE("/api/users/{id}/favorites/soundbites/{soundbite_id}", {
      params: { path: { id: userId, soundbite_id: soundbiteId } },
    }),
};
