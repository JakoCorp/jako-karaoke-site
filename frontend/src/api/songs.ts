import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

export type SongSummary = components["schemas"]["SongSummary"];
export type SongResponse = components["schemas"]["SongResponse"];
export type SongTagKind = components["schemas"]["SongTagKind"];
export type SongImageInfo = components["schemas"]["SongImageInfo"];
export type SongImageKind = components["schemas"]["SongImageKind"];
export type SongSort = components["schemas"]["SongSort"];
export type SongSortDir = components["schemas"]["SortDir"];

/** Query parameters accepted by the songs list endpoint. */
export type SongListParams = SearchPaginationParams & {
  sort?: SongSort;
  sort_dir?: SongSortDir;
};

export const SONG_IMAGE_KINDS = ["cover_art"] as const satisfies readonly SongImageKind[];

export const SONG_TAG_KINDS = [
  "genre",
  "source",
  "language",
  "misc",
] as const satisfies readonly SongTagKind[];

/** Song endpoints. */
export const songsApi = {
  /** Returns a paginated list of songs. */
  list: (params?: SongListParams) => api.GET("/api/songs", { params: { query: params } }),

  /** Returns a single song by ID. */
  get: (id: string) => api.GET("/api/songs/{id}", { params: { path: { id } } }),

  /** Returns the lyrics for a song, or 404 if none are set. */
  getLyrics: (id: string) => api.GET("/api/songs/{id}/lyrics", { params: { path: { id } } }),

  /** Creates a new song. */
  create: (body: components["schemas"]["CreateSongRequest"]) => api.POST("/api/songs", { body }),

  /** Updates a song by ID. */
  update: (id: string, body: components["schemas"]["UpdateSongRequest"]) =>
    api.PUT("/api/songs/{id}", { params: { path: { id } }, body }),

  /** Deletes a song by ID. */
  delete: (id: string) => api.DELETE("/api/songs/{id}", { params: { path: { id } } }),

  /** Uploads and links an image to a song. */
  uploadImage: (id: string, file: File, kind: string, credits?: string | null) =>
    api.POST("/api/songs/{id}/images", {
      params: { path: { id } },
      body: {
        file: "",
        kind,
        credits: credits ?? null,
      } satisfies components["schemas"]["ImageUpload"],
      bodySerializer: () => {
        const form = new FormData();
        form.append("file", file);
        form.append("kind", kind);
        if (credits) form.append("credits", credits);
        return form;
      },
    }),

  /** Updates the kind of an image linked to a song. */
  updateImageKind: (id: string, imageId: string, kind: SongImageKind) =>
    api.PATCH("/api/songs/{id}/images/{image_id}", {
      params: { path: { id, image_id: imageId } },
      body: { kind } satisfies components["schemas"]["UpdateSongImageRequest"],
    }),

  /** Removes an image link from a song, deleting the image file if no other resource references it. */
  deleteImage: (id: string, imageId: string) =>
    api.DELETE("/api/songs/{id}/images/{image_id}", {
      params: { path: { id, image_id: imageId } },
    }),
};
