import { Popover } from "@base-ui/react";
import { CheckIcon, HeartIcon, PlusIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { useAddToPlaylist, useUserPlaylistsForPicker } from "@/hooks/api/playlists";
import { useAuthStore } from "@/store/auth";

type Props = {
  performanceId: string;
  children: React.ReactElement;
};

export function PlaylistPickerPopover({ performanceId, children }: Props) {
  const user = useAuthStore((s) => s.user);
  const [filter, setFilter] = useState("");
  const [addedIds, setAddedIds] = useState<Set<string>>(new Set());

  const { data: allPlaylists = [], isLoading } = useUserPlaylistsForPicker(user?.id ?? null);
  const addToPlaylist = useAddToPlaylist();

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
          setAddedIds((prev) => new Set([...prev, playlistId]));
        },
      },
    );
  }

  return (
    <Popover.Root onOpenChange={() => setFilter("")}>
      <Popover.Trigger render={children} />
      <Popover.Portal>
        <Popover.Positioner side="bottom" align="start" sideOffset={6}>
          <Popover.Popup className="playlist-picker-popup">
            {!user ? (
              <p className="playlist-picker-empty">Sign in to add to playlists.</p>
            ) : (
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
            )}
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
