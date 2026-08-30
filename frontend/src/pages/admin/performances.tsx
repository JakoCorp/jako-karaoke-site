import { Dialog } from "@base-ui/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { artists as artistsApi } from "@/api/artists";
import type { components } from "@/api/generated";
import { performances as performancesApi } from "@/api/performances";
import { songs as songsApi } from "@/api/songs";
import { tags as tagsApi } from "@/api/tags";

import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { resolveTagAssignments } from "./tag-utils";

type PerformanceSummary = components["schemas"]["PerformanceSummary"];
type ArtistResponse = components["schemas"]["ArtistResponse"];
type SongSummary = components["schemas"]["SongSummary"];
type TagResponse = components["schemas"]["TagResponse"];
type PerformanceTagKind = components["schemas"]["PerformanceTagKind"];

const PERFORMANCE_TAG_KINDS: readonly PerformanceTagKind[] = ["instrument", "modifier", "misc"];

function CreatePerformanceDialog({
  open,
  onOpenChange,
  allSongs,
  allArtists,
  allTags,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  allSongs: SongSummary[];
  allArtists: ArtistResponse[];
  allTags: TagResponse[];
}) {
  const [title, setTitle] = useState("");
  const [performanceDate, setPerformanceDate] = useState("");
  const [selectedSongIds, setSelectedSongIds] = useState<string[]>([]);
  const [selectedSingerIds, setSelectedSingerIds] = useState<string[]>([]);
  const [selectedTags, setSelectedTags] = useState<TagAssignment<PerformanceTagKind>[]>([]);
  const [lyrics, setLyrics] = useState("");
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
      const { error: apiError } = await performancesApi.create({
        performance_date: new Date(performanceDate).toISOString(),
        song_ids: selectedSongIds,
        singer_ids: selectedSingerIds,
        tags: resolvedTags,
        title: title.trim() !== "" ? title.trim() : null,
        lyrics: lyrics.trim() !== "" ? lyrics.trim() : null,
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["admin", "performances"] });
      void queryClient.invalidateQueries({ queryKey: ["admin", "tags"] });
      onOpenChange(false);
    },
    onError: () => {
      setError("Failed to create performance.");
    },
  });

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setTitle("");
      setPerformanceDate("");
      setSelectedSongIds([]);
      setSelectedSingerIds([]);
      setSelectedTags([]);
      setLyrics("");
      setError(null);
    }
    onOpenChange(nextOpen);
  }

  function toggleSong(songId: string) {
    setSelectedSongIds((previous) =>
      previous.includes(songId) ? previous.filter((id) => id !== songId) : [...previous, songId],
    );
  }

  function toggleSinger(singerId: string) {
    setSelectedSingerIds((previous) =>
      previous.includes(singerId)
        ? previous.filter((id) => id !== singerId)
        : [...previous, singerId],
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

  function changeTagKind(key: string, kind: PerformanceTagKind) {
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
          <Dialog.Title className="admin-dialog-title">New performance</Dialog.Title>
          <form
            className="admin-dialog-form"
            onSubmit={(event) => {
              event.preventDefault();
              createMutation.mutate();
            }}
          >
            <div className="form-field">
              <label className="form-label" htmlFor="perf-date">
                Performance date
              </label>
              <input
                id="perf-date"
                className="form-input"
                type="datetime-local"
                value={performanceDate}
                onChange={(event) => {
                  setPerformanceDate(event.target.value);
                }}
                required
              />
            </div>
            <div className="form-field">
              <label className="form-label" htmlFor="perf-title">
                Title (optional)
              </label>
              <input
                id="perf-title"
                className="form-input"
                type="text"
                value={title}
                onChange={(event) => {
                  setTitle(event.target.value);
                }}
              />
            </div>
            <ItemPicker
              label="Songs"
              allItems={allSongs}
              selectedIds={selectedSongIds}
              onToggle={toggleSong}
              searchPlaceholder="Search songs…"
              getLabel={(song) => song.title}
              renderDropdownItem={(song) => (
                <>
                  {song.title}
                  {song.artists.length > 0 && (
                    <span className="admin-item-sub">
                      {" "}
                      — {song.artists.map((artist) => artist.name).join(", ")}
                    </span>
                  )}
                </>
              )}
            />
            <ItemPicker
              label="Singers"
              allItems={allArtists}
              selectedIds={selectedSingerIds}
              onToggle={toggleSinger}
              searchPlaceholder="Search singers…"
              getLabel={(artist) => artist.name}
            />
            <TagPicker
              allTags={allTags}
              selected={selectedTags}
              kinds={PERFORMANCE_TAG_KINDS}
              onAddExisting={addExistingTag}
              onAddPending={addPendingTag}
              onRemove={removeTag}
              onKindChange={changeTagKind}
            />
            <div className="form-field">
              <label className="form-label" htmlFor="perf-lyrics">
                Lyrics (optional)
              </label>
              <textarea
                id="perf-lyrics"
                className="form-input admin-textarea"
                value={lyrics}
                onChange={(event) => {
                  setLyrics(event.target.value);
                }}
                rows={4}
              />
            </div>
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
                disabled={createMutation.isPending || performanceDate === ""}
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

function formatDate(isoString: string): string {
  return new Date(isoString).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function PerformancesAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedPerformance, setSelectedPerformance] = useState<PerformanceSummary | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const queryClient = useQueryClient();

  const { data: performancePage, isLoading: performancesLoading } = useQuery({
    queryKey: ["admin", "performances"],
    queryFn: async () => {
      const { data, error } = await performancesApi.list({ per_page: 100 });
      if (error) throw error;
      return data;
    },
  });

  const { data: performanceDetail } = useQuery({
    queryKey: ["admin", "performances", selectedPerformance?.id],
    queryFn: async () => {
      const { data, error } = await performancesApi.get(selectedPerformance!.id);
      if (error) throw error;
      return data;
    },
    enabled: !!selectedPerformance,
  });

  const { data: allSongs } = useQuery({
    queryKey: ["admin", "songs", "picker"],
    queryFn: async () => {
      const { data, error } = await songsApi.list({ per_page: 200 });
      if (error) throw error;
      return data?.items ?? [];
    },
    enabled: createOpen,
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

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await performancesApi.delete(id);
      if (error) throw error;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["admin", "performances"] });
      setSelectedPerformance(null);
      setConfirmDelete(false);
    },
  });

  const allPerformances = performancePage?.items ?? [];
  const filtered = allPerformances.filter((performance) => {
    const titleMatch = (performance.title ?? "").toLowerCase().includes(searchInput.toLowerCase());
    const singerMatch = performance.singers.some((singer) =>
      singer.name.toLowerCase().includes(searchInput.toLowerCase()),
    );
    return titleMatch || singerMatch;
  });

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input"
            style={{ flex: 1 }}
            placeholder="Search performances…"
            value={searchInput}
            onChange={(event) => {
              setSearchInput(event.target.value);
            }}
            aria-label="Search performances"
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
          {performancesLoading && <li className="admin-empty">Loading…</li>}
          {!performancesLoading && filtered.length === 0 && (
            <li className="admin-empty">No performances found.</li>
          )}
          {filtered.map((performance) => (
            <li key={performance.id}>
              <button
                className={`admin-user-item${selectedPerformance?.id === performance.id ? " admin-user-item--active" : ""}`}
                onClick={() => {
                  setSelectedPerformance(performance);
                  setConfirmDelete(false);
                }}
              >
                <div className="admin-item-title">
                  {performance.title ?? formatDate(performance.performance_date)}
                </div>
                {performance.singers.length > 0 && (
                  <div className="admin-item-sub">
                    {performance.singers.map((singer) => singer.name).join(", ")}
                  </div>
                )}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="admin-panel">
        {selectedPerformance ? (
          <>
            <div className="admin-panel-header">
              <h3 className="admin-panel-title">
                {selectedPerformance.title ?? formatDate(selectedPerformance.performance_date)}
              </h3>
              {confirmDelete ? (
                <div className="admin-tag-confirm">
                  <span className="admin-empty">Delete?</span>
                  <button
                    className="btn btn-primary"
                    onClick={() => {
                      deleteMutation.mutate(selectedPerformance.id);
                    }}
                    disabled={deleteMutation.isPending}
                  >
                    Yes
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={() => {
                      setConfirmDelete(false);
                    }}
                  >
                    No
                  </button>
                </div>
              ) : (
                <button
                  className="btn btn-secondary"
                  onClick={() => {
                    setConfirmDelete(true);
                  }}
                >
                  Delete
                </button>
              )}
            </div>

            {performanceDetail && (
              <>
                <div className="admin-detail-section">
                  <span className="admin-detail-label">Date</span>
                  <span className="admin-empty">
                    {formatDate(performanceDetail.performance_date)}
                  </span>
                </div>
                {performanceDetail.songs.length > 0 && (
                  <div className="admin-detail-section">
                    <span className="admin-detail-label">Songs</span>
                    <div className="admin-pills">
                      {performanceDetail.songs.map((song) => (
                        <span key={song.id} className="admin-pill admin-pill--display">
                          {song.title}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                {performanceDetail.singers.length > 0 && (
                  <div className="admin-detail-section">
                    <span className="admin-detail-label">Singers</span>
                    <div className="admin-pills">
                      {performanceDetail.singers.map((singer) => (
                        <span key={singer.id} className="admin-pill admin-pill--display">
                          {singer.name}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
                {performanceDetail.tags.length > 0 && (
                  <div className="admin-detail-section">
                    <span className="admin-detail-label">Tags</span>
                    <div className="admin-pills">
                      {performanceDetail.tags.map((tag) => (
                        <span key={tag.id} className="admin-pill admin-pill--display">
                          {tag.name}
                          <span className="admin-pill-kind">{tag.kind}</span>
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}
          </>
        ) : (
          <p className="admin-empty">Select a performance to view details.</p>
        )}
      </div>

      <CreatePerformanceDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        allSongs={allSongs ?? []}
        allArtists={allArtists ?? []}
        allTags={allTags ?? []}
      />
    </div>
  );
}
