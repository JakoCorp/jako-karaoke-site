import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import {
  SONG_IMAGE_KINDS,
  SONG_TAG_KINDS,
  songsApi,
  type SongImageInfo,
  type SongImageKind,
  type SongSummary,
  type SongTagKind,
} from "@/api/songs";
import { tagsApi } from "@/api/tags";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useArtists } from "@/hooks/api/artists";
import { songKeys, useSong } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";
import { applyAll } from "@/lib/staging";

import { ImageEditSection } from "../shared/image-edit-section";
import { ItemPicker, TagPicker, type TagAssignment } from "../shared/pickers";
import { resolveTagAssignments } from "../shared/tag-utils";

export function SongDetailPanel({
  song,
  onClose,
}: {
  song: SongSummary | null;
  onClose: () => void;
}) {
  const isCreating = song === null;
  const [isEditing, setIsEditing] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editArtistIds, setEditArtistIds] = useState<string[]>([]);
  const [editTags, setEditTags] = useState<TagAssignment<SongTagKind>[]>([]);
  const [stagingAddImages, setStagingAddImages] = useState<File[]>([]);
  const [pendingRemoveIds, setPendingRemoveIds] = useState<Set<string>>(new Set());
  const [pendingImageKindChanges, setPendingImageKindChanges] = useState<
    Map<string, SongImageKind>
  >(new Map());
  const [formError, setFormError] = useState<string | null>(null);

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
      const { data, error: apiError } = await songsApi.create({
        title: editTitle.trim(),
        artist_ids: editArtistIds,
        tags: resolvedTags,
      });
      if (apiError) throw apiError;
      if (!data) throw new Error("Song creation returned no data.");
      await applyAll(stagingAddImages, (file) => songsApi.uploadImage(data.id, file, "cover_art"));
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
        tags: resolvedTags,
      });
      if (apiError) throw apiError;
      await applyAll(stagingAddImages, (file) => songsApi.uploadImage(song.id, file, "cover_art"));
      await applyAll(pendingImageKindChanges, ([imageId, kind]) =>
        songsApi.updateImageKind(song.id, imageId, kind),
      );
      await applyAll(pendingRemoveIds, (id) => songsApi.deleteImage(song.id, id));
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: songKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setStagingAddImages([]);
      setPendingRemoveIds(new Set());
      setPendingImageKindChanges(new Map());
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
    setStagingAddImages([]);
    setPendingRemoveIds(new Set());
    setPendingImageKindChanges(new Map());
    setFormError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setStagingAddImages([]);
    setPendingRemoveIds(new Set());
    setPendingImageKindChanges(new Map());
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
    const existingImages = isEditing ? (songDetail?.images ?? []) : [];
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
          <ImageEditSection
            existingImages={existingImages}
            pendingRemoveIds={pendingRemoveIds}
            pendingKindChanges={pendingImageKindChanges}
            stagingFiles={stagingAddImages}
            kinds={SONG_IMAGE_KINDS}
            onFileSelect={(file) => {
              setStagingAddImages((prev) => [...prev, file]);
            }}
            onRemoveExisting={(id) => {
              setPendingRemoveIds((prev) => new Set([...prev, id]));
            }}
            onChangeExistingKind={(id, kind) => {
              setPendingImageKindChanges((prev) => new Map(prev).set(id, kind));
            }}
            onRemoveStaged={(index) => {
              setStagingAddImages((prev) => prev.filter((_, i) => i !== index));
            }}
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
        <div className="admin-tag-confirm">
          <button className="btn btn-secondary" onClick={startEditing} disabled={!songDetail}>
            Edit
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => {
              setDeleteOpen(true);
            }}
          >
            Delete
          </button>
        </div>
      </div>
      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={(nextOpen) => {
          setDeleteOpen(nextOpen);
          if (!nextOpen) deleteMutation.reset();
        }}
        onConfirm={() => {
          deleteMutation.mutate();
        }}
        variant="danger"
        isPending={deleteMutation.isPending}
        error={deleteMutation.isError ? "Failed to delete song." : null}
      />

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
          {songDetail.images.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Images</span>
              <div className="admin-image-list">
                {songDetail.images.map((image: SongImageInfo) => (
                  <div key={image.id} className="admin-image-item">
                    <img src={image.public_url} alt={image.kind} />
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </>
  );
}
