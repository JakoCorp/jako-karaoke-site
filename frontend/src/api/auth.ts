import { api } from "./client";
import type { components } from "./generated";

/** Auth endpoints: session management and credential based login/register. */
export const auth = {
  /** Returns the currently authenticated user, or null if no session is active. */
  me: () => api.GET("/auth/me", {}),

  /** Creates a new user account and issues a session. */
  register: (body: components["schemas"]["RegisterRequest"]) =>
    api.POST("/auth/register", { body }),

  /** Verifies credentials and issues a session. */
  login: (body: components["schemas"]["LoginRequest"]) => api.POST("/auth/login", { body }),

  /** Revokes the current session and clears the session cookie. */
  logout: () => api.POST("/auth/logout", {}),
};
