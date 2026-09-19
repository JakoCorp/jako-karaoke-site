import { XIcon } from "@phosphor-icons/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import {
  ARTIST_IMAGE_KINDS,
  ARTIST_LINK_KINDS,
  artistsApi,
  type ArtistImageInfo,
  type ArtistImageKind,
  type ArtistLinkKind,
  type ArtistSummary,
} from "@/api/artists";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { artistKeys, useArtist } from "@/hooks/api/artists";
import { applyAll } from "@/lib/staging";

import { ImageEditSection } from "../shared/image-edit-section";

type LinkDraft = { url: string; kind: ArtistLinkKind; label: string };

function isArtistLinkKind(value: string): value is ArtistLinkKind {
  return (ARTIST_LINK_KINDS as readonly string[]).includes(value);
}

function linkDraftFromInfo(link: { url: string; kind: string; label?: string | null }): LinkDraft {
  return {
    url: link.url,
    kind: isArtistLinkKind(link.kind) ? link.kind : "other",
    label: link.label ?? "",
  };
}

export function ArtistDetailPanel({
  artist,
  onClose,
}: {
  artist: ArtistSummary | null;
  onClose: () => void;
}) {
  const isCreating = artist === null;
  const [isEditing, setIsEditing] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [editName, setEditName] = useState("");
  const [editDescription, setEditDescription] = useState("");
  const [editLinks, setEditLinks] = useState<LinkDraft[]>([]);
  const [stagingAddImages, setStagingAddImages] = useState<File[]>([]);
  const [pendingRemoveIds, setPendingRemoveIds] = useState<Set<string>>(new Set());
  const [pendingImageKindChanges, setPendingImageKindChanges] = useState<
    Map<string, ArtistImageKind>
  >(new Map());
  const [formError, setFormError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const { data: artistDetail } = useArtist(artist?.id ?? "", !isCreating);

  const createMutation = useMutation({
    mutationFn: async () => {
      const validLinks = editLinks.filter((link) => link.url.trim() !== "");
      const { data, error: apiError } = await artistsApi.create({
        name: editName.trim(),
        description: editDescription.trim() !== "" ? editDescription.trim() : null,
        links: validLinks.map((link) => ({
          url: link.url.trim(),
          kind: link.kind,
          label: link.label.trim() !== "" ? link.label.trim() : null,
        })),
      });
      if (apiError) throw apiError;
      if (!data) throw new Error("Artist creation returned no data.");
      await applyAll(stagingAddImages, (file) => artistsApi.uploadImage(data.id, file, "avatar"));
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: artistKeys.all() });
      onClose();
    },
    onError: () => {
      setFormError("Failed to create artist.");
    },
  });

  const updateMutation = useMutation({
    mutationFn: async () => {
      if (!artist) return;
      const validLinks = editLinks.filter((link) => link.url.trim() !== "");
      const { error: apiError } = await artistsApi.update(artist.id, {
        name: editName.trim(),
        description: editDescription.trim() !== "" ? editDescription.trim() : null,
        links: validLinks.map((link) => ({
          url: link.url.trim(),
          kind: link.kind,
          label: link.label.trim() !== "" ? link.label.trim() : null,
        })),
      });
      if (apiError) throw apiError;
      await applyAll(stagingAddImages, (file) => artistsApi.uploadImage(artist.id, file, "avatar"));
      await applyAll(pendingImageKindChanges, ([imageId, kind]) =>
        artistsApi.updateImageKind(artist.id, imageId, kind),
      );
      await applyAll(pendingRemoveIds, (id) => artistsApi.deleteImage(artist.id, id));
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: artistKeys.all() });
      setStagingAddImages([]);
      setPendingRemoveIds(new Set());
      setPendingImageKindChanges(new Map());
      setIsEditing(false);
      setFormError(null);
    },
    onError: () => {
      setFormError("Failed to update artist.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      if (!artist) return;
      const { error: apiError } = await artistsApi.delete(artist.id);
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: artistKeys.all() });
      onClose();
    },
  });

  function startEditing() {
    if (!artistDetail) return;
    setEditName(artistDetail.name);
    setEditDescription(artistDetail.description ?? "");
    setEditLinks(artistDetail.links.map(linkDraftFromInfo));
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

  function addLink() {
    setEditLinks((previous) => [...previous, { url: "", kind: "other", label: "" }]);
  }

  function removeLink(index: number) {
    setEditLinks((previous) => previous.filter((_, i) => i !== index));
  }

  function updateLink(index: number, patch: Partial<LinkDraft>) {
    setEditLinks((previous) =>
      previous.map((link, i) => (i === index ? { ...link, ...patch } : link)),
    );
  }

  if (isCreating || isEditing) {
    const isPending = isCreating ? createMutation.isPending : updateMutation.isPending;
    const existingImages = isEditing ? (artistDetail?.images ?? []) : [];
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{isCreating ? "New artist" : artist.name}</h3>
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
              disabled={isPending || editName.trim() === ""}
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
            <label className="form-label" htmlFor="artist-edit-name">
              Name
            </label>
            <input
              id="artist-edit-name"
              className="form-input"
              type="text"
              value={editName}
              onChange={(event) => {
                setEditName(event.target.value);
              }}
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="artist-edit-description">
              Description (optional)
            </label>
            <textarea
              id="artist-edit-description"
              className="form-input admin-textarea"
              value={editDescription}
              onChange={(event) => {
                setEditDescription(event.target.value);
              }}
              rows={3}
            />
          </div>
          <div className="form-field">
            <span className="form-label">Links</span>
            <div className="admin-link-list">
              {editLinks.map((link, index) => (
                <div key={index} className="admin-link-card">
                  <div className="admin-link-card-header">
                    <select
                      className="admin-kind-select"
                      value={link.kind}
                      onChange={(event) => {
                        if (isArtistLinkKind(event.target.value)) {
                          updateLink(index, { kind: event.target.value });
                        }
                      }}
                    >
                      {ARTIST_LINK_KINDS.map((kind) => (
                        <option key={kind} value={kind}>
                          {kind}
                        </option>
                      ))}
                    </select>
                    <button
                      className="btn btn-secondary"
                      type="button"
                      onClick={() => {
                        removeLink(index);
                      }}
                    >
                      <XIcon />
                    </button>
                  </div>
                  <input
                    className="form-input"
                    type="url"
                    placeholder="URL"
                    value={link.url}
                    onChange={(event) => {
                      updateLink(index, { url: event.target.value });
                    }}
                  />
                  <input
                    className="form-input"
                    type="text"
                    placeholder="Label (optional)"
                    value={link.label}
                    onChange={(event) => {
                      updateLink(index, { label: event.target.value });
                    }}
                  />
                </div>
              ))}
              <button className="btn btn-secondary" type="button" onClick={addLink}>
                Add link
              </button>
            </div>
          </div>
          <ImageEditSection
            existingImages={existingImages}
            pendingRemoveIds={pendingRemoveIds}
            pendingKindChanges={pendingImageKindChanges}
            stagingFiles={stagingAddImages}
            kinds={ARTIST_IMAGE_KINDS}
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
        <h3 className="admin-panel-title">{artist.name}</h3>
        <div className="admin-tag-confirm">
          <button className="btn btn-secondary" onClick={startEditing} disabled={!artistDetail}>
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
        error={deleteMutation.isError ? "Failed to delete artist." : null}
      />

      {artistDetail && (
        <div className="admin-panel-scroll">
          {artistDetail.description && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Description</span>
              <p className="admin-empty">{artistDetail.description}</p>
            </div>
          )}
          {artistDetail.links.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Links</span>
              <div className="admin-link-list">
                {artistDetail.links.map((link) => (
                  <div key={link.id} className="admin-link-row">
                    <span className="admin-pill-kind">{link.kind}</span>
                    <a className="admin-link-url" href={link.url} target="_blank" rel="noreferrer">
                      {link.label ?? link.url}
                    </a>
                    {link.label && <span className="admin-link-label">{link.url}</span>}
                  </div>
                ))}
              </div>
            </div>
          )}
          {artistDetail.images.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Images</span>
              <div className="admin-image-list">
                {artistDetail.images.map((image: ArtistImageInfo) => (
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
