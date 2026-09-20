import { CheckIcon, HeartIcon, PlusIcon } from "@phosphor-icons/react";
import { useState } from "react";

import {
  useAddToPlaylist,
  usePlaylistsContaining,
  useUserPlaylistsForPicker,
} from "@/hooks/api/playlists";
import { useAuthStore } from "@/store/auth";

interface Props {
  performanceId: string;
}

export function PlaylistPickerContent({ performanceId }: Props) {
  const user = useAuthStore((s) => s.user);
  const [filter, setFilter] = useState("");
  const [sessionAddedIds, setSessionAddedIds] = useState<Set<string>>(new Set());

  const { data: allPlaylists = [], isLoading } = useUserPlaylistsForPicker(user?.id ?? null);
  const { data: existingIds = new Set<string>() } = usePlaylistsContaining(
    user?.id ?? null,
    performanceId,
  );
  const addToPlaylist = useAddToPlaylist();

  const addedIds = new Set([...existingIds, ...sessionAddedIds]);
  const filtered = allPlaylists.filter((p) => p.title.toLowerCase().includes(filter.toLowerCase()));
  const sorted = [
    ...filtered.filter((p) => !addedIds.has(p.id)),
    ...filtered.filter((p) => addedIds.has(p.id)),
  ];

  function handleAdd(playlistId: string) {
    addToPlaylist.mutate(
      { playlistId, performanceIds: [performanceId] },
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
      <div className="playlist-picker-list">
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
                onClick={() => handleAdd(playlist.id)}
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
      </div>
    </>
  );
}
