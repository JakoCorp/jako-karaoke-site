import { useQuery } from "@tanstack/react-query";

import { filtersApi } from "@/api/filters";
import type { FilterSubject } from "@/api/filters";

const CATALOG_STALE_MS = 5 * 60 * 1000;

export const filterCatalogKeys = {
  all: () => ["filter-catalog"] as const,
  subject: (subject: FilterSubject) => ["filter-catalog", subject] as const,
};

/** Loads the values that can be used as filters. */
export function useFilterCatalog(subject: FilterSubject) {
  return useQuery({
    queryKey: filterCatalogKeys.subject(subject),
    queryFn: async () => {
      const { data, error } = await filtersApi.catalog(subject);
      if (error) throw error;
      return data.groups;
    },
    staleTime: CATALOG_STALE_MS,
  });
}
