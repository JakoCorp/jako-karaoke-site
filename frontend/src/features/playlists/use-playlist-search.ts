import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";

import type { PlaylistSortField } from "@/api/playlists";
import { useDebounced } from "@/hooks/use-debounced";
import { createStateCodec } from "@/lib/url-state";

export type PlaylistSortOption = `${PlaylistSortField}-${"asc" | "desc"}`;

interface PlaylistState {
  q?: string;
  sort?: PlaylistSortField;
  sort_dir?: "asc" | "desc";
}

export const PLAYLIST_SORT_OPTIONS: {
  value: PlaylistSortOption;
  label: string;
  params: Partial<PlaylistState>;
}[] = [
  { value: "created-desc", label: "Newest", params: {} },
  { value: "created-asc", label: "Oldest", params: { sort_dir: "asc" } },
  { value: "title-asc", label: "A-Z", params: { sort: "title", sort_dir: "asc" } },
  { value: "title-desc", label: "Z-A", params: { sort: "title" } },
  {
    value: "performance_count-desc",
    label: "Most tracks",
    params: { sort: "performance_count" },
  },
  {
    value: "performance_count-asc",
    label: "Fewest tracks",
    params: { sort: "performance_count", sort_dir: "asc" },
  },
];

const playlistStateCodec = createStateCodec<PlaylistState>({
  compact: (state) => {
    const compact: PlaylistState = {};
    if (state.q) compact.q = state.q;
    if (state.sort && state.sort !== "created") compact.sort = state.sort;
    if (state.sort_dir && state.sort_dir !== "desc") compact.sort_dir = state.sort_dir;
    return compact;
  },
  validate: (parsed) => {
    const state: PlaylistState = {};
    if (typeof parsed.q === "string" && parsed.q) state.q = parsed.q;
    if (
      parsed.sort === "created" ||
      parsed.sort === "title" ||
      parsed.sort === "performance_count"
    ) {
      state.sort = parsed.sort;
    }
    if (parsed.sort_dir === "asc" || parsed.sort_dir === "desc") state.sort_dir = parsed.sort_dir;
    return state;
  },
});

function withState(prev: URLSearchParams, partial: Partial<PlaylistState>): URLSearchParams {
  const current = playlistStateCodec.decode(prev.get("query") ?? "");
  const next = playlistStateCodec.toParam({ ...current, ...partial });
  const params = new URLSearchParams(prev);
  if (next) {
    params.set("query", next);
  } else {
    params.delete("query");
  }
  return params;
}

/**
 * Search text and sort of a playlist list page, kept in the encoded `query` search param.
 *
 * `params` can be passed straight to the infinite playlist queries.
 */
export function usePlaylistSearch() {
  const [searchParams, setSearchParams] = useSearchParams();
  const playlistState = playlistStateCodec.decode(searchParams.get("query") ?? "");

  const q = playlistState.q ?? "";
  const sort = playlistState.sort ?? "created";
  const sortDir = playlistState.sort_dir ?? "desc";

  const [inputValue, setInputValue] = useState(q);
  const [prevQ, setPrevQ] = useState(q);
  const debouncedInput = useDebounced(inputValue);

  if (q !== prevQ) {
    setPrevQ(q);
    setInputValue(q);
  }

  useEffect(() => {
    setSearchParams((prev) => withState(prev, { q: debouncedInput.trim() || undefined }), {
      replace: true,
    });
  }, [debouncedInput, setSearchParams]);

  function setSortOption(value: PlaylistSortOption) {
    const option = PLAYLIST_SORT_OPTIONS.find((opt) => opt.value === value);
    if (option) {
      setSearchParams((prev) =>
        withState(prev, { sort: undefined, sort_dir: undefined, ...option.params }),
      );
    }
  }

  return {
    inputValue,
    setInputValue,
    sortOption: `${sort}-${sortDir}` satisfies PlaylistSortOption,
    setSortOption,
    params: { q: q || undefined, sort, sort_dir: sortDir },
  };
}
