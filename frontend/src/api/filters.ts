import { api } from "./client";
import type { components } from "./generated";

export type FilterField = components["schemas"]["FilterField"];
export type FilterSubject = components["schemas"]["FilterSubject"];
export type CatalogValue = components["schemas"]["CatalogValue"];
export type CatalogGroup = components["schemas"]["CatalogGroup"];

/** Filter endpoints. */
export const filtersApi = {
  /** Returns the values that can be used as filters for performances or songs. */
  catalog: (subject: FilterSubject) =>
    api.GET("/api/filters/catalog", { params: { query: { subject } } }),
};
