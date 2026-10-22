import { CheckIcon, HeartIcon, PlusIcon } from "@phosphor-icons/react";
import { useRef, useState } from "react";

import {
  useAddToPlaylist,
  useFavoritesForPicker,
  useInfinitePickerPlaylists,
  usePlaylistsContaining,
} from "@/hooks/api/playlists";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";
import { useAuthStore } from "@/store/auth";

interface Props {
  performanceId: string;
}

export function PlaylistPickerContent({ performanceId }: Props) {
  const user = useAuthStore((s) => s.user);
  const [filter, setFilter] = useState("");
  const [sessionAddedIds, setSessionAddedIds] = useState<Set<string>>(new Set());

  const debouncedFilter = useDebounced(filter);
  const q = debouncedFilter.trim() || undefined;
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const { data: favorites = [], isLoading: isLoadingFavorites } = useFavoritesForPicker(
    user?.id ?? null,
    q,
  );
  const {
    data: pages,
    isLoading: isLoadingPlaylists,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfinitePickerPlaylists(user?.id ?? null, q);
  const sentinelRef = useInfiniteSentinel({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });
  const { data: existingIds = new Set<string>() } = usePlaylistsContaining(
    user?.id ?? null,
    performanceId,
  );
  const addToPlaylist = useAddToPlaylist();

  const isLoading = isLoadingFavorites || isLoadingPlaylists;
  const addedIds = new Set([...existingIds, ...sessionAddedIds]);
  const playlists = [...favorites, ...(pages?.pages.flatMap((page) => page.items) ?? [])];
  const sorted = [
    ...playlists.filter((p) => !addedIds.has(p.id)),
    ...playlists.filter((p) => addedIds.has(p.id)),
  ];

  function handleAdd(playlistId: string, playlistTitle: string) {
    addToPlaylist.mutate(
      { playlistId, playlistTitle, performanceIds: [performanceId] },
      {
        onSuccess: () => {
          setSessionAddedIds((prev) => new Set([...prev, playlistId]));
        },
      },
    );
  }

  if (!user) {
    return <p className="playlist-picker-empty">Sign in to add to playlists.</p>;
  }

  return (
    <>
      <div className="playlist-picker-filter">
        <input
          type="search"
          className="form-input playlist-picker-filter-input"
          placeholder="Filter playlists…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          aria-label="Filter playlists"
        />
      </div>
      <div ref={scrollContainerRef} className="playlist-picker-list">
        {isLoading && <p className="playlist-picker-empty">Loading…</p>}
        {!isLoading && sorted.length === 0 && (
          <p className="playlist-picker-empty">No playlists found.</p>
        )}
        {sorted.map((playlist) => {
          const added = addedIds.has(playlist.id);
          return (
            <div key={playlist.id} className="playlist-picker-item">
              <span className="playlist-picker-item-name">{playlist.title}</span>
              <button
                type="button"
                className="playlist-picker-add-btn"
                aria-label={added ? "Added" : `Add to ${playlist.title}`}
                disabled={added || addToPlaylist.isPending}
                onClick={() => handleAdd(playlist.id, playlist.title)}
              >
                {added ? (
                  <CheckIcon size={14} />
                ) : playlist.kind === "favorites" ? (
                  <HeartIcon size={14} />
                ) : (
                  <PlusIcon size={14} />
                )}
              </button>
            </div>
          );
        })}
        <div ref={sentinelRef} />
        {isFetchingNextPage && <p className="playlist-picker-empty">Loading more…</p>}
      </div>
    </>
  );
}
