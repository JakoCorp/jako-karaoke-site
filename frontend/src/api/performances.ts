import { api } from "./client";
import type { components } from "./generated";
import type { SearchPaginationParams } from "./types";

export type PerformanceSummary = components["schemas"]["PerformanceSummary"];
export type PerformanceResponse = components["schemas"]["PerformanceResponse"];
export type PerformanceTagKind = components["schemas"]["PerformanceTagKind"];
export type AudioInfo = components["schemas"]["AudioInfo"];
export type AudioKind = components["schemas"]["AudioKind"];
export type VideoInfo = components["schemas"]["VideoInfo"];
export type VideoKind = components["schemas"]["VideoKind"];
export type PerformanceSortField = components["schemas"]["PerformanceSort"];
export type PerformanceSortDir = components["schemas"]["SortDir"];

/** Query parameters accepted by the performances list endpoint. */
export type PerformanceListParams = SearchPaginationParams & {
  sort?: PerformanceSortField;
  sort_dir?: PerformanceSortDir;
};

export const PERFORMANCE_TAG_KINDS = [
  "instrument",
  "modifier",
  "misc",
] as const satisfies readonly PerformanceTagKind[];

export const AUDIO_KINDS = ["primary", "misc"] as const satisfies readonly AudioKind[];

export const VIDEO_KINDS = ["clip", "vod", "misc"] as const satisfies readonly VideoKind[];

/** Performance endpoints. */
export const performancesApi = {
  /** Returns a paginated, optionally filtered list of performances. */
  list: (params?: PerformanceListParams) =>
    api.GET("/api/performances", { params: { query: params } }),

  /** Returns a single performance by ID. */
  get: (id: string) => api.GET("/api/performances/{id}", { params: { path: { id } } }),

  /**
   * Returns lyrics for a performance.
   * Falls back to the linked song's lyrics if no performance-specific override is set.
   * Returns 404 if neither the performance nor the song has lyrics.
   */
  getLyrics: (id: string) => api.GET("/api/performances/{id}/lyrics", { params: { path: { id } } }),

  /** Creates a new performance. */
  create: (body: components["schemas"]["CreatePerformanceRequest"]) =>
    api.POST("/api/performances", { body }),

  /** Updates a performance by ID. */
  update: (id: string, body: components["schemas"]["UpdatePerformanceRequest"]) =>
    api.PUT("/api/performances/{id}", { params: { path: { id } }, body }),

  /** Deletes a performance by ID. */
  delete: (id: string) => api.DELETE("/api/performances/{id}", { params: { path: { id } } }),

  /** Uploads an audio file for a performance with the given kind. */
  uploadAudio: (id: string, file: File, kind: AudioKind) =>
    api.POST("/api/performances/{id}/audio", {
      params: { path: { id } },
      body: { file: "", kind } satisfies components["schemas"]["AudioUpload"],
      bodySerializer: () => {
        const form = new FormData();
        form.append("file", file);
        form.append("kind", kind);
        return form;
      },
    }),

  /** Updates the kind of an audio record attached to a performance. */
  updateAudioKind: (id: string, audioId: string, kind: AudioKind) =>
    api.PATCH("/api/performances/{id}/audio/{audio_id}", {
      params: { path: { id, audio_id: audioId } },
      body: { kind } satisfies components["schemas"]["UpdateAudioKindRequest"],
    }),

  /** Removes an audio file from a performance. */
  deleteAudio: (id: string, audioId: string) =>
    api.DELETE("/api/performances/{id}/audio/{audio_id}", {
      params: { path: { id, audio_id: audioId } },
    }),

  /** Uploads a video file for a performance with the given kind. */
  uploadVideo: (id: string, file: File, kind: VideoKind) =>
    api.POST("/api/performances/{id}/video", {
      params: { path: { id } },
      body: { file: "", kind } satisfies components["schemas"]["VideoUpload"],
      bodySerializer: () => {
        const form = new FormData();
        form.append("file", file);
        form.append("kind", kind);
        return form;
      },
    }),

  /** Updates the kind of a video record attached to a performance. */
  updateVideoKind: (id: string, videoId: string, kind: VideoKind) =>
    api.PATCH("/api/performances/{id}/video/{video_id}", {
      params: { path: { id, video_id: videoId } },
      body: { kind } satisfies components["schemas"]["UpdateVideoKindRequest"],
    }),

  /** Removes a video file from a performance. */
  deleteVideo: (id: string, videoId: string) =>
    api.DELETE("/api/performances/{id}/video/{video_id}", {
      params: { path: { id, video_id: videoId } },
    }),
};
