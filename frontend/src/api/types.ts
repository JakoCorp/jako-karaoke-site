/** Shared pagination query parameters for list endpoints. */
export type PaginationParams = {
  page?: number;
  per_page?: number;
};

/** Pagination query parameters with an optional text search filter. */
export type SearchPaginationParams = PaginationParams & {
  q?: string;
};
