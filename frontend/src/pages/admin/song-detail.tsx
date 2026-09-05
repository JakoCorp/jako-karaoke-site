import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { SONG_TAG_KINDS, songsApi, type SongSummary, type SongTagKind } from "@/api/songs";
import { tagsApi } from "@/api/tags";
import { useArtists } from "@/hooks/api/artists";
import { songKeys, useSong } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";

import { ItemPicker, TagPicker, type TagAssignment } from "./pickers";
import { resolveTagAssignments } from "./tag-utils";

export function SongDetailPanel({
  song,
  onClose,
}: {
  song: SongSummary | null;
  onClose: () => void;
}) {
  const isCreating = song === null;
  const [isEditing, setIsEditing] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editArtistIds, setEditArtistIds] = useState<string[]>([]);
  const [editTags, setEditTags] = useState<TagAssignment<SongTagKind>[]>([]);
  const [formError, setFormError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const queryClient = useQueryClient();
  const isFormOpen = isCreating || isEditing;

  const { data: songDetail } = useSong(song?.id ?? "", !isCreating);
  const { data: allArtists } = useArtists({ per_page: 200 }, isFormOpen);
  const { data: allTags } = useTags(isFormOpen);

  const createMutation = useMutation({
    mutationFn: async () => {
      const resolvedTags = await resolveTagAssignments(editTags, async (name) => {
        const { data, error: tagError } = await tagsApi.create({ name });
        if (tagError) throw tagError;
        if (!data) throw new Error("Tag creation returned no data.");
        return data.id;
      });
      const { error: apiError } = await songsApi.create({
        title: editTitle.trim(),
        artist_ids: editArtistIds,
        image_ids: [],
        tags: resolvedTags,
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: songKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      onClose();
    },
    onError: () => {
      setFormError("Failed to create song.");
    },
  });

  const updateMutation = useMutation({
    mutationFn: async () => {
      if (!song) return;
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
      setFormError(null);
    },
    onError: () => {
      setFormError("Failed to update song.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      if (!song) return;
      const { error: apiError } = await songsApi.delete(song.id);
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: songKeys.all() });
      onClose();
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
    setFormError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setIsEditing(false);
    setFormError(null);
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

  if (isFormOpen) {
    const isPending = isCreating ? createMutation.isPending : updateMutation.isPending;
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{isCreating ? "New song" : song.title}</h3>
          <div className="admin-tag-confirm">
            <button
              className="btn btn-secondary"
              type="button"
              onClick={isCreating ? onClose : cancelEditing}
            >
              Cancel
            </button>
            <button
              className="btn btn-primary"
              type="button"
              disabled={isPending || editTitle.trim() === ""}
              onClick={() => {
                if (isCreating) {
                  createMutation.mutate();
                } else {
                  updateMutation.mutate();
                }
              }}
            >
              {isPending ? (isCreating ? "Creating…" : "Saving…") : isCreating ? "Create" : "Save"}
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
          {formError !== null && <p className="form-error">{formError}</p>}
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
                  <span key={artist.id} className="admin-pill-display">
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
                  <span key={tag.id} className="admin-pill-display">
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
