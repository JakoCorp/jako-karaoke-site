import type { components, operations } from "./generated";

/** Sort direction accepted by the sortable list endpoints. */
export type SortDir = components["schemas"]["SortDir"];

/** Query parameters of a generated operation, documented by the API spec. */
export type QueryOf<Operation extends keyof operations> = NonNullable<
  operations[Operation]["parameters"]["query"]
>;

/** List query parameters without the paging fields, for infinite queries that choose the page. */
export type WithoutPaging<T> = Omit<T, "page" | "per_page">;
