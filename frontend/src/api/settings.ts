import { api } from "./client";
import type { components } from "./generated";

export type UserSettings = components["schemas"]["UserSettingsResponse"];
export type DownloadSettings = components["schemas"]["DownloadSettings"];

/** Download preferences used until a user saves their own. Mirrors the server defaults. */
export const DEFAULT_DOWNLOAD_SETTINGS: DownloadSettings = {
  filename_template: "{date} {stream_number} #{performance_number} {title}",
  include_cover_art: true,
  include_lyrics: false,
  include_date: true,
  include_singers: true,
  include_original_artists: false,
};

/** Endpoints for the authenticated user's preferences. */
export const settingsApi = {
  /** Returns the user's settings, or the server defaults when none are saved. */
  get: () => api.GET("/auth/me/settings", {}),

  /** Replaces all of the user's settings. */
  update: (body: components["schemas"]["UpdateUserSettingsRequest"]) =>
    api.PUT("/auth/me/settings", { body }),
};
