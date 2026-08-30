import { api } from "./client";
import type { components } from "./generated";

export type TagResponse = components["schemas"]["TagResponse"];

/** Tag endpoints. */
export const tags = {
  /** Returns all tags. */
  list: () => api.GET("/api/tags", {}),

  /** Creates a new tag. */
  create: (body: components["schemas"]["CreateTagRequest"]) => api.POST("/api/tags", { body }),

  /** Deletes a tag by ID. */
  delete: (id: string) => api.DELETE("/api/tags/{id}", { params: { path: { id } } }),
};
