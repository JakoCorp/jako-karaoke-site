import { api } from "./client";
import type { components } from "./generated";

export type UserSummary = components["schemas"]["UserSummary"];

/** User endpoints. */
export const usersApi = {
  /** Searches users by optional username substring. */
  search: (q?: string) => api.GET("/api/users", { params: { query: { q } } }),

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
};
