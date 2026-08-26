import { api } from "./client";

/** Tag endpoints. */
export const tags = {
  /** Returns all tags. */
  list: () => api.GET("/api/tags", {}),
};
