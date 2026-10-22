import { useState } from "react";
import { useNavigate } from "react-router";

import { useDeletePlaylist, useInfiniteUserPlaylists } from "@/hooks/api/playlists";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";
import { useAuthStore } from "@/store/auth";

import { CreatePlaylistDialog } from "../components/create-playlist-dialog";
import { PlaylistGrid } from "../components/grid";
import { PlaylistSearchControls } from "../components/search-controls";
import { usePlaylistSearch } from "../use-playlist-search";

export function MyPlaylistsPage() {
  const user = useAuthStore((s) => s.user);
  const [dialogOpen, setDialogOpen] = useState(false);
  const navigate = useNavigate();

  const search = usePlaylistSearch();
  const deletePlaylist = useDeletePlaylist();

  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfiniteUserPlaylists(user?.id ?? null, search.params);
  const sentinelRef = useInfiniteSentinel({ hasNextPage, isFetchingNextPage, fetchNextPage });

  if (!user) {
    return <div className="page-empty">Sign in to see your playlists.</div>;
  }

  const total = data?.pages[0]?.total ?? 0;
  const items = data?.pages.flatMap((page) => page.items) ?? [];

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-header-row">
          <div className="playlist-page-title">My Playlists</div>
          <button type="button" className="btn btn-primary" onClick={() => setDialogOpen(true)}>
            New playlist
          </button>
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
        <PlaylistGrid
          playlists={items}
          showVisibility
          emptyMessage="You have no playlists."
          onDelete={(id) => deletePlaylist.mutate(id)}
        />
      )}
      <div ref={sentinelRef} />
      {isFetchingNextPage && <div className="page-empty">Loading more…</div>}
      <CreatePlaylistDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onCreated={(playlist) => {
          void navigate(`/playlist/${playlist.id}`);
        }}
      />
    </div>
  );
}
