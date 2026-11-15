import { useState } from "react";

import { useDebounced } from "@/hooks/use-debounced";

/** Search text of a picker with its debounced `query`, undefined while blank. */
export function usePickerSearch() {
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounced(search);
  const query = debouncedSearch.trim() || undefined;
  return {
    search,
    onSearchChange: setSearch,
    query,
    isDebouncing: search.trim() !== (query ?? ""),
  };
}
