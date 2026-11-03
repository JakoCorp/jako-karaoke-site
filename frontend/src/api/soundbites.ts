import { multipartSerializer } from "@/lib/multipart";

import { api } from "./client";
import type { components } from "./generated";
import type { QueryOf } from "./types";

/** Query parameters of the soundbite list endpoint. */
export type SoundbiteListParams = QueryOf<"list_soundbites">;

export type SoundbiteSortField = components["schemas"]["SoundbiteSort"];

export type SoundbiteResponse = components["schemas"]["SoundbiteResponse"];
export type SoundbitePlayer = components["schemas"]["SoundbitePlayer"];

/** Soundbite endpoints. */
export const soundbitesApi = {
  /** Returns a paginated list of soundbites, newest first. */
  list: (params?: SoundbiteListParams) => api.GET("/api/soundbites", { params: { query: params } }),

  /** Creates a soundbite without audio. */
  create: (body: components["schemas"]["CreateSoundbiteRequest"]) =>
    api.POST("/api/soundbites", { body }),

  /** Replaces all fields of a soundbite except its audio. */
  update: (id: string, body: components["schemas"]["UpdateSoundbiteRequest"]) =>
    api.PUT("/api/soundbites/{id}", { params: { path: { id } }, body }),

  /** Deletes a soundbite and its audio file when nothing else uses it. */
  delete: (id: string) => api.DELETE("/api/soundbites/{id}", { params: { path: { id } } }),

  /** Uploads the audio file of a soundbite, replacing the previous one. */
  uploadAudio: (id: string, file: File) =>
    api.PUT("/api/soundbites/{id}/audio", {
      params: { path: { id } },
      body: { file: "" } satisfies components["schemas"]["SoundbiteAudioUpload"],
      bodySerializer: multipartSerializer({ file }),
    }),

  /** Returns a paginated list of the users who played a soundbite, most plays first. */
  listPlayers: (id: string, page: number, perPage: number) =>
    api.GET("/api/soundbites/{id}/plays", {
      params: { path: { id }, query: { page, per_page: perPage } },
    }),

  /** Opens a play session for the signed in user. */
  startPlay: (id: string) => api.POST("/api/soundbites/{id}/plays", { params: { path: { id } } }),

  /** Completes a play session once enough of the soundbite was listened to. */
  completePlay: (id: string, playId: string) =>
    api.POST("/api/soundbites/{id}/plays/{play_id}/complete", {
      params: { path: { id, play_id: playId } },
    }),
};
