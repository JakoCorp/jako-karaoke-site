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
};
