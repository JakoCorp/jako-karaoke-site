import { useRef, useState } from "react";

import { type SongSummary } from "@/api/songs";
import { useInfiniteSongs } from "@/hooks/api/songs";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";

import { SongDetailPanel } from "./detail";

export function SongsAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedSong, setSelectedSong] = useState<SongSummary | null>(null);
  const [creating, setCreating] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const scrollContainerRef = useRef<HTMLUListElement>(null);

  const {
    data: songPages,
    isLoading: songsLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteSongs({ q: debouncedQuery || undefined });
  const sentinelRef = useInfiniteSentinel<HTMLLIElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });

  const songs = songPages?.pages.flatMap((page) => page.items) ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input flex-1"
            placeholder="Search songs…"
            value={searchInput}
            onChange={(event) => {
              setSearchInput(event.target.value);
            }}
            aria-label="Search songs"
          />
          <button
            className="btn btn-primary"
            onClick={() => {
              setSelectedSong(null);
              setCreating(true);
            }}
          >
            New
          </button>
        </div>
        <ul ref={scrollContainerRef} className="admin-user-list">
          {songsLoading && <li className="admin-empty">Loading…</li>}
          {!songsLoading && songs.length === 0 && <li className="admin-empty">No songs found.</li>}
          {songs.map((song) => (
            <li key={song.id}>
              <button
                className={
                  selectedSong?.id === song.id && !creating
                    ? "admin-user-item admin-user-item--active"
                    : "admin-user-item"
                }
                onClick={() => {
                  setCreating(false);
                  setSelectedSong(song);
                }}
              >
                <div className="admin-item-title">{song.title}</div>
                {song.artists.length > 0 && (
                  <div className="admin-item-sub">
                    {song.artists.map((artist) => artist.name).join(", ")}
                  </div>
                )}
              </button>
            </li>
          ))}
          <li ref={sentinelRef} />
          {isFetchingNextPage && <li className="admin-empty">Loading more…</li>}
        </ul>
      </div>

      <div className="admin-panel">
        {creating ? (
          <SongDetailPanel key="new" song={null} onClose={() => setCreating(false)} />
        ) : selectedSong ? (
          <SongDetailPanel
            key={selectedSong.id}
            song={selectedSong}
            onClose={() => setSelectedSong(null)}
          />
        ) : (
          <p className="admin-empty">Select a song to view details.</p>
        )}
      </div>
    </div>
  );
}
