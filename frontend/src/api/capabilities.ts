import { api } from "./client";

/** Capability reference endpoints. */
export const capabilitiesApi = {
  /** Returns all known capability strings. */
  list: () => api.GET("/api/capabilities"),
};
