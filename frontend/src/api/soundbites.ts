import { multipartSerializer } from "@/lib/multipart";

import { api } from "./client";
import type { components } from "./generated";
import type { QueryOf } from "./types";

/** Query parameters of the soundbite list endpoint. */
export type SoundbiteListParams = QueryOf<"list_soundbites">;

export type SoundbiteResponse = components["schemas"]["SoundbiteResponse"];

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
};
