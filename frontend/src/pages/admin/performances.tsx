import { Dialog } from "@base-ui/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { type ArtistSummary } from "@/api/artists";
import {
  PERFORMANCE_TAG_KINDS,
  performancesApi,
  type PerformanceSummary,
  type PerformanceTagKind,
} from "@/api/performances";
import { type SongSummary } from "@/api/songs";
import { tagsApi, type TagResponse } from "@/api/tags";
import { useArtists } from "@/hooks/api/artists";
import { performanceKeys, usePerformances } from "@/hooks/api/performances";
import { useSongs } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";
import { useDebounced } from "@/hooks/use-debounced";
import { formatDate } from "@/lib/format";

import { PerformanceDetailPanel } from "./performance-detail";
import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { resolveTagAssignments } from "./tag-utils";

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
  allArtists: ArtistSummary[];
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
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
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

export function PerformancesAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedPerformance, setSelectedPerformance] = useState<PerformanceSummary | null>(null);
  const [createOpen, setCreateOpen] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const { data: performancePage, isLoading: performancesLoading } = usePerformances({
    per_page: 100,
    q: debouncedQuery || undefined,
  });

  const { data: allSongs } = useSongs({ per_page: 200 }, createOpen);
  const { data: allArtists } = useArtists({ per_page: 200 }, createOpen);
  const { data: allTags } = useTags(createOpen);

  const performances = performancePage?.items ?? [];

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
          {!performancesLoading && performances.length === 0 && (
            <li className="admin-empty">No performances found.</li>
          )}
          {performances.map((performance) => (
            <li key={performance.id}>
              <button
                className={`admin-user-item${selectedPerformance?.id === performance.id ? " admin-user-item--active" : ""}`}
                onClick={() => {
                  setSelectedPerformance(performance);
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
          <PerformanceDetailPanel
            key={selectedPerformance.id}
            performance={selectedPerformance}
            onDeleted={() => {
              setSelectedPerformance(null);
            }}
          />
        ) : (
          <p className="admin-empty">Select a performance to view details.</p>
        )}
      </div>

      <CreatePerformanceDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        allSongs={allSongs?.items ?? []}
        allArtists={allArtists?.items ?? []}
        allTags={allTags ?? []}
      />
    </div>
  );
}
