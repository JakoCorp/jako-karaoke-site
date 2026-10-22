import { useRef, useState } from "react";

import { type ArtistSummary } from "@/api/artists";
import { useInfiniteArtists } from "@/hooks/api/artists";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";

import { ArtistDetailPanel } from "./detail";

export function ArtistsAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedArtist, setSelectedArtist] = useState<ArtistSummary | null>(null);
  const [creating, setCreating] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const scrollContainerRef = useRef<HTMLUListElement>(null);

  const {
    data: artistPages,
    isLoading: artistsLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteArtists({ q: debouncedQuery || undefined });
  const sentinelRef = useInfiniteSentinel<HTMLLIElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });

  const artists = artistPages?.pages.flatMap((page) => page.items) ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input flex-1"
            placeholder="Search artists…"
            value={searchInput}
            onChange={(event) => {
              setSearchInput(event.target.value);
            }}
            aria-label="Search artists"
          />
          <button
            className="btn btn-primary"
            onClick={() => {
              setSelectedArtist(null);
              setCreating(true);
            }}
          >
            New
          </button>
        </div>
        <ul ref={scrollContainerRef} className="admin-user-list">
          {artistsLoading && <li className="admin-empty">Loading…</li>}
          {!artistsLoading && artists.length === 0 && (
            <li className="admin-empty">No artists found.</li>
          )}
          {artists.map((artist) => (
            <li key={artist.id}>
              <button
                className={
                  selectedArtist?.id === artist.id && !creating
                    ? "admin-user-item admin-user-item--active"
                    : "admin-user-item"
                }
                onClick={() => {
                  setCreating(false);
                  setSelectedArtist(artist);
                }}
              >
                <div className="admin-item-title">{artist.name}</div>
                {artist.description && (
                  <div className="admin-item-sub admin-item-sub--clamped">{artist.description}</div>
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
          <ArtistDetailPanel key="new" artist={null} onClose={() => setCreating(false)} />
        ) : selectedArtist ? (
          <ArtistDetailPanel
            key={selectedArtist.id}
            artist={selectedArtist}
            onClose={() => setSelectedArtist(null)}
          />
        ) : (
          <p className="admin-empty">Select an artist to view details.</p>
        )}
      </div>
    </div>
  );
}
