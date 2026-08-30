import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import {
  PERFORMANCE_TAG_KINDS,
  performances as performancesApi,
  type PerformanceSummary,
  type PerformanceTagKind,
} from "@/api/performances";
import { tags as tagsApi } from "@/api/tags";
import { useArtists } from "@/hooks/api/artists";
import { performanceKeys, usePerformance } from "@/hooks/api/performances";
import { useSongs } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";
import { formatDate, isoToDatetimeLocal } from "@/lib/format";

import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { resolveTagAssignments } from "./tag-utils";

export function PerformanceDetailPanel({
  performance,
  onDeleted,
}: {
  performance: PerformanceSummary;
  onDeleted: () => void;
}) {
  const [isEditing, setIsEditing] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editDate, setEditDate] = useState("");
  const [editSongIds, setEditSongIds] = useState<string[]>([]);
  const [editSingerIds, setEditSingerIds] = useState<string[]>([]);
  const [editTags, setEditTags] = useState<TagAssignment<PerformanceTagKind>[]>([]);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const { data: performanceDetail } = usePerformance(performance.id);
  const { data: allSongs } = useSongs({ per_page: 200 }, isEditing);
  const { data: allArtists } = useArtists({ per_page: 200 }, isEditing);
  const { data: allTags } = useTags(isEditing);

  const updateMutation = useMutation({
    mutationFn: async () => {
      const resolvedTags = await resolveTagAssignments(editTags, async (name) => {
        const { data, error: tagError } = await tagsApi.create({ name });
        if (tagError) throw tagError;
        if (!data) throw new Error("Tag creation returned no data.");
        return data.id;
      });
      const { error: apiError } = await performancesApi.update(performance.id, {
        performance_date: new Date(editDate).toISOString(),
        song_ids: editSongIds,
        singer_ids: editSingerIds,
        tags: resolvedTags,
        title: editTitle.trim() !== "" ? editTitle.trim() : null,
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setIsEditing(false);
      setUpdateError(null);
    },
    onError: () => {
      setUpdateError("Failed to update performance.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      const { error: apiError } = await performancesApi.delete(performance.id);
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      onDeleted();
    },
    onError: () => {
      setDeleteError("Failed to delete performance.");
    },
  });

  function startEditing() {
    if (!performanceDetail) return;
    setEditTitle(performanceDetail.title ?? "");
    setEditDate(isoToDatetimeLocal(performanceDetail.performance_date));
    setEditSongIds(performanceDetail.songs.map((song) => song.id));
    setEditSingerIds(performanceDetail.singers.map((singer) => singer.id));
    setEditTags(
      performanceDetail.tags.map((tag) => ({
        type: "existing" as const,
        tagId: tag.id,
        kind: PERFORMANCE_TAG_KINDS.find((k) => k === tag.kind) ?? "misc",
      })),
    );
    setUpdateError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setIsEditing(false);
    setUpdateError(null);
  }

  function toggleSong(songId: string) {
    setEditSongIds((previous) =>
      previous.includes(songId) ? previous.filter((id) => id !== songId) : [...previous, songId],
    );
  }

  function toggleSinger(singerId: string) {
    setEditSingerIds((previous) =>
      previous.includes(singerId)
        ? previous.filter((id) => id !== singerId)
        : [...previous, singerId],
    );
  }

  function addExistingTag(tagId: string) {
    setEditTags((previous) => [...previous, { type: "existing", tagId, kind: "misc" }]);
  }

  function addPendingTag(name: string) {
    setEditTags((previous) => [...previous, { type: "pending", pendingName: name, kind: "misc" }]);
  }

  function removeTag(key: string) {
    setEditTags((previous) =>
      previous.filter((a) => (a.type === "existing" ? a.tagId !== key : a.pendingName !== key)),
    );
  }

  function changeTagKind(key: string, kind: PerformanceTagKind) {
    setEditTags((previous) =>
      previous.map((a) => {
        if (a.type === "existing" && a.tagId === key) return { ...a, kind };
        if (a.type === "pending" && a.pendingName === key) return { ...a, kind };
        return a;
      }),
    );
  }

  const displayTitle = performance.title ?? formatDate(performance.performance_date);

  if (isEditing) {
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{displayTitle}</h3>
          <div className="admin-tag-confirm">
            <button className="btn btn-secondary" type="button" onClick={cancelEditing}>
              Cancel
            </button>
            <button
              className="btn btn-primary"
              type="button"
              disabled={updateMutation.isPending || editDate === ""}
              onClick={() => {
                updateMutation.mutate();
              }}
            >
              {updateMutation.isPending ? "Saving…" : "Save"}
            </button>
          </div>
        </div>
        <div className="admin-panel-scroll">
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-date">
              Performance date
            </label>
            <input
              id="perf-edit-date"
              className="form-input"
              type="datetime-local"
              value={editDate}
              onChange={(event) => {
                setEditDate(event.target.value);
              }}
              required
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-title">
              Title (optional)
            </label>
            <input
              id="perf-edit-title"
              className="form-input"
              type="text"
              value={editTitle}
              onChange={(event) => {
                setEditTitle(event.target.value);
              }}
            />
          </div>
          <ItemPicker
            label="Songs"
            allItems={allSongs?.items ?? []}
            selectedIds={editSongIds}
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
            allItems={allArtists?.items ?? []}
            selectedIds={editSingerIds}
            onToggle={toggleSinger}
            searchPlaceholder="Search singers…"
            getLabel={(artist) => artist.name}
          />
          <TagPicker
            allTags={allTags ?? []}
            selected={editTags}
            kinds={PERFORMANCE_TAG_KINDS}
            onAddExisting={addExistingTag}
            onAddPending={addPendingTag}
            onRemove={removeTag}
            onKindChange={changeTagKind}
          />
          {updateError !== null && <p className="form-error">{updateError}</p>}
        </div>
      </>
    );
  }

  return (
    <>
      <div className="admin-panel-header">
        <h3 className="admin-panel-title">{displayTitle}</h3>
        {confirmDelete ? (
          <div className="admin-tag-confirm">
            <span className="admin-empty">Delete?</span>
            <button
              className="btn btn-primary"
              onClick={() => {
                deleteMutation.mutate();
              }}
              disabled={deleteMutation.isPending}
            >
              Yes
            </button>
            <button
              className="btn btn-secondary"
              onClick={() => {
                setConfirmDelete(false);
                setDeleteError(null);
              }}
            >
              No
            </button>
          </div>
        ) : (
          <div className="admin-tag-confirm">
            <button
              className="btn btn-secondary"
              onClick={startEditing}
              disabled={!performanceDetail}
            >
              Edit
            </button>
            <button
              className="btn btn-secondary"
              onClick={() => {
                setConfirmDelete(true);
              }}
            >
              Delete
            </button>
          </div>
        )}
      </div>

      {deleteError !== null && <p className="form-error">{deleteError}</p>}

      {performanceDetail && (
        <div className="admin-panel-scroll">
          <div className="admin-detail-section">
            <span className="admin-detail-label">Date</span>
            <span className="admin-empty">{formatDate(performanceDetail.performance_date)}</span>
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
        </div>
      )}
    </>
  );
}
