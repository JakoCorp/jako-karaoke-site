import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { components } from "@/api/generated";
import { songs as songsApi } from "@/api/songs";
import { tags as tagsApi } from "@/api/tags";
import { useArtists } from "@/hooks/api/artists";
import { songKeys, useSong } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";

import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { resolveTagAssignments } from "./tag-utils";

type SongSummary = components["schemas"]["SongSummary"];
type SongTagKind = components["schemas"]["SongTagKind"];

const SONG_TAG_KINDS: readonly SongTagKind[] = ["genre", "source", "language", "misc"];

export function SongDetailPanel({ song, onDeleted }: { song: SongSummary; onDeleted: () => void }) {
  const [isEditing, setIsEditing] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editArtistIds, setEditArtistIds] = useState<string[]>([]);
  const [editTags, setEditTags] = useState<TagAssignment<SongTagKind>[]>([]);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const { data: songDetail } = useSong(song.id);
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
      const { error: apiError } = await songsApi.update(song.id, {
        title: editTitle.trim(),
        artist_ids: editArtistIds,
        image_ids: songDetail?.images.map((image) => image.id) ?? [],
        tags: resolvedTags,
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: songKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setIsEditing(false);
      setUpdateError(null);
    },
    onError: () => {
      setUpdateError("Failed to update song.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      const { error: apiError } = await songsApi.delete(song.id);
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: songKeys.all() });
      onDeleted();
    },
    onError: () => {
      setDeleteError("Failed to delete song.");
    },
  });

  function startEditing() {
    if (!songDetail) return;
    setEditTitle(songDetail.title);
    setEditArtistIds(songDetail.artists.map((artist) => artist.id));
    setEditTags(
      songDetail.tags.map((tag) => ({
        type: "existing" as const,
        tagId: tag.id,
        kind: SONG_TAG_KINDS.find((k) => k === tag.kind) ?? "misc",
      })),
    );
    setUpdateError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setIsEditing(false);
    setUpdateError(null);
  }

  function toggleArtist(artistId: string) {
    setEditArtistIds((previous) =>
      previous.includes(artistId)
        ? previous.filter((id) => id !== artistId)
        : [...previous, artistId],
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

  function changeTagKind(key: string, kind: SongTagKind) {
    setEditTags((previous) =>
      previous.map((a) => {
        if (a.type === "existing" && a.tagId === key) return { ...a, kind };
        if (a.type === "pending" && a.pendingName === key) return { ...a, kind };
        return a;
      }),
    );
  }

  if (isEditing) {
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{song.title}</h3>
          <div className="admin-tag-confirm">
            <button className="btn btn-secondary" type="button" onClick={cancelEditing}>
              Cancel
            </button>
            <button
              className="btn btn-primary"
              type="button"
              disabled={updateMutation.isPending || editTitle.trim() === ""}
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
            <label className="form-label" htmlFor="song-edit-title">
              Title
            </label>
            <input
              id="song-edit-title"
              className="form-input"
              type="text"
              value={editTitle}
              onChange={(event) => {
                setEditTitle(event.target.value);
              }}
            />
          </div>
          <ItemPicker
            label="Artists"
            allItems={allArtists?.items ?? []}
            selectedIds={editArtistIds}
            onToggle={toggleArtist}
            searchPlaceholder="Search artists…"
            getLabel={(artist) => artist.name}
          />
          <TagPicker
            allTags={allTags ?? []}
            selected={editTags}
            kinds={SONG_TAG_KINDS}
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
        <h3 className="admin-panel-title">{song.title}</h3>
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
            <button className="btn btn-secondary" onClick={startEditing} disabled={!songDetail}>
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

      {songDetail && (
        <div className="admin-panel-scroll">
          {songDetail.artists.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Artists</span>
              <div className="admin-pills">
                {songDetail.artists.map((artist) => (
                  <span key={artist.id} className="admin-pill admin-pill--display">
                    {artist.name}
                  </span>
                ))}
              </div>
            </div>
          )}
          {songDetail.tags.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Tags</span>
              <div className="admin-pills">
                {songDetail.tags.map((tag) => (
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
