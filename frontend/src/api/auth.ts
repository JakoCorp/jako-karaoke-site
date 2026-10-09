import { api } from "./client";
import type { components } from "./generated";

export type MeResponse = components["schemas"]["MeResponse"];

/** The currently authenticated user as the client models it. */
export interface AuthUser {
  readonly id: string;
  readonly username: string;
  readonly avatarUrl: string | null;
  readonly usernameChangeableAt: string | null;
  readonly capabilities: readonly string[];
}

/** Maps an API `MeResponse` to the client side `AuthUser`. */
export function toAuthUser(me: MeResponse): AuthUser {
  return {
    id: me.id,
    username: me.username,
    avatarUrl: me.avatar_url ?? null,
    usernameChangeableAt: me.username_changeable_at ?? null,
    capabilities: me.capabilities,
  };
}

/** Auth endpoints: session management and OAuth claim. */
export const authApi = {
  /** Returns the currently authenticated user, or null if no session is active. */
  me: () => api.GET("/auth/me", {}),

  /** Returns 204 if a valid pending OAuth signup session exists, 404 otherwise. */
  checkPending: () => api.GET("/auth/pending", {}),

  /**
   * Completes an OAuth signup by claiming a username.
   * Requires an active `oauth_pending` cookie from the OAuth callback.
   */
  claim: (body: components["schemas"]["ClaimRequest"]) => api.POST("/auth/claim", { body }),

  /** Changes the current user's username, subject to the 30 day cooldown. */
  updateMe: (username: string) =>
    api.PATCH("/auth/me", {
      body: { username } satisfies components["schemas"]["UpdateMeRequest"],
    }),

  /** Replaces the current user's avatar with an uploaded image. */
  uploadAvatar: (file: File) =>
    api.PUT("/auth/me/avatar", {
      body: { file: "" } satisfies components["schemas"]["AvatarUpload"],
      bodySerializer: () => {
        const form = new FormData();
        form.append("file", file);
        return form;
      },
    }),

  /** Revokes the current session and clears the session cookie. */
  logout: () => api.POST("/auth/logout", {}),

  /**
   * Issues a session as the seeded dev admin user.
   * Only available when the backend is running with `DEV_AUTH=true`.
   */
  devLogin: () => api.GET("/auth/dev-login", {}),
};
