import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

/** Fields available for sorting the artists list. */
export type ArtistSortField = "name" | "song_count";

/** Query parameters for the artist list endpoint. */
export type ArtistListParams = SearchPaginationParams & {
  /** Field to sort by. Defaults to `name`. */
  sort?: ArtistSortField;
  /** Sort direction. Defaults to `asc`. */
  sort_dir?: "asc" | "desc";
};

export type ArtistSummary = components["schemas"]["ArtistSummary"];
export type ArtistResponse = components["schemas"]["ArtistResponse"];
export type ArtistLinkInfo = components["schemas"]["ArtistLinkInfo"];
export type ArtistImageInfo = components["schemas"]["ArtistImageInfo"];
export type ArtistImageKind = components["schemas"]["ArtistImageKind"];

export const ARTIST_IMAGE_KINDS = ["avatar"] as const satisfies readonly ArtistImageKind[];

export const ARTIST_LINK_KINDS = [
  "youtube",
  "website",
  "x",
  "instagram",
  "twitch",
  "other",
] as const;
export type ArtistLinkKind = (typeof ARTIST_LINK_KINDS)[number];

/** Artist endpoints. */
export const artistsApi = {
  /** Returns a paginated list of artists. */
  list: (params?: ArtistListParams) => api.GET("/api/artists", { params: { query: params } }),

  /** Returns a single artist by ID. */
  get: (id: string) => api.GET("/api/artists/{id}", { params: { path: { id } } }),

  /** Creates a new artist. */
  create: (body: components["schemas"]["CreateArtistRequest"]) =>
    api.POST("/api/artists", { body }),

  /** Replaces all fields of an artist. */
  update: (id: string, body: components["schemas"]["UpdateArtistRequest"]) =>
    api.PUT("/api/artists/{id}", { params: { path: { id } }, body }),

  /** Deletes an artist by ID. */
  delete: (id: string) => api.DELETE("/api/artists/{id}", { params: { path: { id } } }),

  /** Uploads and links an image to an artist. */
  uploadImage: (id: string, file: File, kind: string, credits?: string | null) =>
    api.POST("/api/artists/{id}/images", {
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

  /** Updates the kind of an image linked to an artist. */
  updateImageKind: (id: string, assetId: string, kind: ArtistImageKind) =>
    api.PATCH("/api/artists/{id}/images/{asset_id}", {
      params: { path: { id, asset_id: assetId } },
      body: { kind } satisfies components["schemas"]["UpdateArtistImageRequest"],
    }),

  /** Removes an image link from an artist, deleting the image file if no other resource references it. */
  deleteImage: (id: string, assetId: string) =>
    api.DELETE("/api/artists/{id}/images/{asset_id}", {
      params: { path: { id, asset_id: assetId } },
    }),
};
