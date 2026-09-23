import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";

import { useDeletePlaylist, useInfiniteUserPlaylists } from "@/hooks/api/playlists";
import { useDebounced } from "@/hooks/use-debounced";
import { useAuthStore } from "@/store/auth";

import { CreatePlaylistDialog } from "../components/create-playlist-dialog";
import { PlaylistGrid } from "../components/grid";

export function MyPlaylistsPage() {
  const user = useAuthStore((s) => s.user);
  const [dialogOpen, setDialogOpen] = useState(false);
  const navigate = useNavigate();

  const [searchParams, setSearchParams] = useSearchParams();
  const q = searchParams.get("q") ?? "";

  const [inputValue, setInputValue] = useState(q);
  const [prevQ, setPrevQ] = useState(q);
  const debouncedInput = useDebounced(inputValue);
  const sentinelRef = useRef<HTMLDivElement>(null);

  if (q !== prevQ) {
    setPrevQ(q);
    setInputValue(q);
  }

  useEffect(() => {
    setSearchParams(
      (prev) => {
        const params = new URLSearchParams(prev);
        const trimmed = debouncedInput.trim();
        if (trimmed) {
          params.set("q", trimmed);
        } else {
          params.delete("q");
        }
        return params;
      },
      { replace: true },
    );
  }, [debouncedInput, setSearchParams]);

  const deletePlaylist = useDeletePlaylist();

  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfiniteUserPlaylists(user?.id ?? null, q || undefined);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasNextPage && !isFetchingNextPage) {
          void fetchNextPage();
        }
      },
      { rootMargin: "200px" },
    );
    if (sentinelRef.current) observer.observe(sentinelRef.current);
    return () => {
      observer.disconnect();
    };
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  if (!user) {
    return <div className="playlist-empty">Sign in to see your playlists.</div>;
  }

  const total = data?.pages[0]?.total ?? 0;
  const items =
    data?.pages.flatMap((page) => page?.items ?? []).filter((p) => p.kind === "user") ?? [];

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-header-row">
          <div className="playlist-page-title">My Playlists</div>
          <button type="button" className="btn btn-primary" onClick={() => setDialogOpen(true)}>
            New playlist
          </button>
        </div>
        <div className="playlist-search-wrapper">
          <MagnifyingGlassIcon size={16} className="playlist-search-icon" />
          <input
            type="search"
            className="form-input playlist-search-input"
            placeholder="Search playlists…"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            aria-label="Search playlists"
          />
        </div>
        {!isLoading && (
          <div className="playlist-page-sub">
            {total.toLocaleString()} {total === 1 ? "playlist" : "playlists"}
          </div>
        )}
      </div>
      {isLoading ? (
        <div className="playlist-empty">Loading…</div>
      ) : (
        <PlaylistGrid
          playlists={items}
          showVisibility
          emptyMessage="You have no playlists."
          onDelete={(id) => deletePlaylist.mutate(id)}
        />
      )}
      <div ref={sentinelRef} />
      {isFetchingNextPage && <div className="playlist-empty">Loading more…</div>}
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
