import { api } from "./client";
import type { components } from "./generated";

/** Auth endpoints: session management and OAuth claim. */
export const auth = {
  /** Returns the currently authenticated user, or null if no session is active. */
  me: () => api.GET("/auth/me", {}),

  /** Returns 204 if a valid pending OAuth signup session exists, 404 otherwise. */
  checkPending: () => api.GET("/auth/pending", {}),

  /**
   * Completes an OAuth signup by claiming a username.
   * Requires an active `oauth_pending` cookie from the OAuth callback.
   */
  claim: (body: components["schemas"]["ClaimRequest"]) => api.POST("/auth/claim", { body }),

  /** Revokes the current session and clears the session cookie. */
  logout: () => api.POST("/auth/logout", {}),
};
