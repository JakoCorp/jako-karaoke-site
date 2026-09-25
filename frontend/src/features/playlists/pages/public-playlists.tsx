import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";

import { useInfinitePublicPlaylists } from "@/hooks/api/playlists";
import { useDebounced } from "@/hooks/use-debounced";

import { PlaylistGrid } from "../components/grid";

export function PublicPlaylistsPage() {
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

  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfinitePublicPlaylists(q || undefined);

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

  const total = data?.pages[0]?.total ?? 0;
  const items =
    data?.pages
      .flatMap((page) => page?.items ?? [])
      .filter((p) => p.is_public && p.kind !== "favorites") ?? [];

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-header-row">
          <div className="playlist-page-title">Public Playlists</div>
        </div>
        <div className="playlist-search-wrapper">
          <MagnifyingGlassIcon size={16} className="playlist-search-icon" />
          <input
            type="search"
            className="form-input search-input"
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
        <PlaylistGrid playlists={items} emptyMessage="No public playlists yet." />
      )}
      <div ref={sentinelRef} />
      {isFetchingNextPage && <div className="playlist-empty">Loading more…</div>}
    </div>
  );
}
