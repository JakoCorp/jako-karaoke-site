import { api } from "./client";
import type { components } from "./generated";
import type { QueryOf } from "./types";

export type UserSummary = components["schemas"]["UserSummary"];
export type UserPlayEntry = components["schemas"]["UserPlayEntry"];

/** Query parameters of the user search endpoint. */
export type UserListParams = QueryOf<"search_users">;

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
  listPlays: (userId: string, page: number, perPage: number) =>
    api.GET("/api/users/{id}/plays", {
      params: { path: { id: userId }, query: { page, per_page: perPage } },
    }),

  /** Removes every counted play of a user. */
  deletePlays: (userId: string) =>
    api.DELETE("/api/users/{id}/plays", { params: { path: { id: userId } } }),

  /** Removes a user's counted plays on one performance. */
  deletePerformancePlays: (userId: string, performanceId: string) =>
    api.DELETE("/api/users/{id}/plays/{performance_id}", {
      params: { path: { id: userId, performance_id: performanceId } },
    }),
};
