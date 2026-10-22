import { useInfinitePublicPlaylists } from "@/hooks/api/playlists";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";

import { PlaylistGrid } from "../components/grid";
import { PlaylistSearchControls } from "../components/search-controls";
import { usePlaylistSearch } from "../use-playlist-search";

export function PublicPlaylistsPage() {
  const search = usePlaylistSearch();

  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfinitePublicPlaylists(search.params);
  const sentinelRef = useInfiniteSentinel({ hasNextPage, isFetchingNextPage, fetchNextPage });

  const total = data?.pages[0]?.total ?? 0;
  const items = data?.pages.flatMap((page) => page.items) ?? [];

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-header-row">
          <div className="playlist-page-title">Public Playlists</div>
        </div>
        <PlaylistSearchControls
          inputValue={search.inputValue}
          onInputChange={search.setInputValue}
          sortOption={search.sortOption}
          onSortChange={search.setSortOption}
        />
        {!isLoading && (
          <div className="playlist-page-sub">
            {total.toLocaleString()} {total === 1 ? "playlist" : "playlists"}
          </div>
        )}
      </div>
      {isLoading ? (
        <div className="page-empty">Loading…</div>
      ) : (
        <PlaylistGrid playlists={items} emptyMessage="No public playlists yet." />
      )}
      <div ref={sentinelRef} />
      {isFetchingNextPage && <div className="page-empty">Loading more…</div>}
    </div>
  );
}
