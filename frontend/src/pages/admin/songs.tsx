import { Dialog } from "@base-ui/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { artists as artistsApi } from "@/api/artists";
import type { components } from "@/api/generated";
import { songs as songsApi } from "@/api/songs";
import { tags as tagsApi } from "@/api/tags";

import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { SongDetailPanel } from "./song-detail";
import { resolveTagAssignments } from "./tag-utils";

type SongSummary = components["schemas"]["SongSummary"];
type ArtistResponse = components["schemas"]["ArtistResponse"];
type TagResponse = components["schemas"]["TagResponse"];
type SongTagKind = components["schemas"]["SongTagKind"];

const SONG_TAG_KINDS: readonly SongTagKind[] = ["genre", "source", "language", "misc"];

function CreateSongDialog({
  open,
  onOpenChange,
  allArtists,
  allTags,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  allArtists: ArtistResponse[];
  allTags: TagResponse[];
}) {
  const [title, setTitle] = useState("");
  const [selectedArtistIds, setSelectedArtistIds] = useState<string[]>([]);
  const [selectedTags, setSelectedTags] = useState<TagAssignment<SongTagKind>[]>([]);
  const [error, setError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const createMutation = useMutation({
    mutationFn: async () => {
      const resolvedTags = await resolveTagAssignments(selectedTags, async (name) => {
        const { data, error: tagError } = await tagsApi.create({ name });
        if (tagError) throw tagError;
        if (!data) throw new Error("Tag creation returned no data.");
        return data.id;
      });
      const { error: apiError } = await songsApi.create({
        title: title.trim(),
        artist_ids: selectedArtistIds,
        image_ids: [],
        tags: resolvedTags,
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["admin", "songs"] });
      void queryClient.invalidateQueries({ queryKey: ["admin", "tags"] });
      onOpenChange(false);
    },
    onError: () => {
      setError("Failed to create song.");
    },
  });

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setTitle("");
      setSelectedArtistIds([]);
      setSelectedTags([]);
      setError(null);
    }
    onOpenChange(nextOpen);
  }

  function toggleArtist(artistId: string) {
    setSelectedArtistIds((previous) =>
      previous.includes(artistId)
        ? previous.filter((id) => id !== artistId)
        : [...previous, artistId],
    );
  }

  function addExistingTag(tagId: string) {
    setSelectedTags((previous) => [...previous, { type: "existing", tagId, kind: "misc" }]);
  }

  function addPendingTag(name: string) {
    setSelectedTags((previous) => [
      ...previous,
      { type: "pending", pendingName: name, kind: "misc" },
    ]);
  }

  function removeTag(key: string) {
    setSelectedTags((previous) =>
      previous.filter((a) => (a.type === "existing" ? a.tagId !== key : a.pendingName !== key)),
    );
  }

  function changeTagKind(key: string, kind: SongTagKind) {
    setSelectedTags((previous) =>
      previous.map((a) => {
        if (a.type === "existing" && a.tagId === key) return { ...a, kind };
        if (a.type === "pending" && a.pendingName === key) return { ...a, kind };
        return a;
      }),
    );
  }

  return (
    <Dialog.Root open={open} onOpenChange={handleOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup dialog-popup--wide">
          <Dialog.Title className="admin-dialog-title">New song</Dialog.Title>
          <form
            className="admin-dialog-form"
            onSubmit={(event) => {
              event.preventDefault();
              createMutation.mutate();
            }}
          >
            <div className="form-field">
              <label className="form-label" htmlFor="song-title">
                Title
              </label>
              <input
                id="song-title"
                className="form-input"
                type="text"
                value={title}
                onChange={(event) => {
                  setTitle(event.target.value);
                }}
                required
              />
            </div>
            <ItemPicker
              label="Artists"
              allItems={allArtists}
              selectedIds={selectedArtistIds}
              onToggle={toggleArtist}
              searchPlaceholder="Search artists…"
              getLabel={(artist) => artist.name}
            />
            <TagPicker
              allTags={allTags}
              selected={selectedTags}
              kinds={SONG_TAG_KINDS}
              onAddExisting={addExistingTag}
              onAddPending={addPendingTag}
              onRemove={removeTag}
              onKindChange={changeTagKind}
            />
            {error !== null && <p className="form-error">{error}</p>}
            <div className="admin-dialog-actions">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  handleOpenChange(false);
                }}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={createMutation.isPending || title.trim() === ""}
              >
                {createMutation.isPending ? "Creating…" : "Create"}
              </button>
            </div>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export function SongsAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selectedSong, setSelectedSong] = useState<SongSummary | null>(null);
  const [createOpen, setCreateOpen] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchInput.trim());
    }, 300);
    return () => {
      clearTimeout(timer);
    };
  }, [searchInput]);

  const { data: songPage, isLoading: songsLoading } = useQuery({
    queryKey: ["admin", "songs", debouncedQuery],
    queryFn: async () => {
      const { data, error } = await songsApi.list({
        per_page: 100,
        q: debouncedQuery || undefined,
      });
      if (error) throw error;
      return data;
    },
  });

  const { data: allArtists } = useQuery({
    queryKey: ["admin", "artists"],
    queryFn: async () => {
      const { data, error } = await artistsApi.list({ per_page: 200 });
      if (error) throw error;
      return data?.items ?? [];
    },
    enabled: createOpen,
  });

  const { data: allTags } = useQuery({
    queryKey: ["admin", "tags"],
    queryFn: async () => {
      const { data, error } = await tagsApi.list();
      if (error) throw error;
      return data ?? [];
    },
    enabled: createOpen,
  });

  const songs = songPage?.items ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input"
            style={{ flex: 1 }}
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
              setCreateOpen(true);
            }}
          >
            New
          </button>
        </div>
        <ul className="admin-user-list">
          {songsLoading && <li className="admin-empty">Loading…</li>}
          {!songsLoading && songs.length === 0 && <li className="admin-empty">No songs found.</li>}
          {songs.map((song) => (
            <li key={song.id}>
              <button
                className={`admin-user-item${selectedSong?.id === song.id ? " admin-user-item--active" : ""}`}
                onClick={() => {
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
        </ul>
      </div>

      <div className="admin-panel">
        {selectedSong ? (
          <SongDetailPanel
            key={selectedSong.id}
            song={selectedSong}
            onDeleted={() => {
              setSelectedSong(null);
            }}
          />
        ) : (
          <p className="admin-empty">Select a song to view details.</p>
        )}
      </div>

      <CreateSongDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        allArtists={allArtists ?? []}
        allTags={allTags ?? []}
      />
    </div>
  );
}
