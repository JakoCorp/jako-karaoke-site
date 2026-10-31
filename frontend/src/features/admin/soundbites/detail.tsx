import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { ensureOk, unwrap } from "@/api/client";
import { soundbitesApi, type SoundbiteResponse } from "@/api/soundbites";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useArtists } from "@/hooks/api/artists";
import { soundbiteKeys } from "@/hooks/api/soundbites";
import { resolveAssetUrl } from "@/lib/asset-url";
import { failureMessage } from "@/lib/error-message";
import { formatDate } from "@/lib/format";

import { assetLabel } from "../components/asset-utils";
import { ItemPicker } from "../components/pickers";
import { SingleAudioField } from "../components/single-audio-field";
import { parseStreamPosition } from "../components/stream-position";

const MAX_TITLE_LENGTH = 256;

function withoutExtension(filename: string): string {
  return filename.replace(/\.[^.]+$/, "");
}

export function SoundbiteDetailPanel({
  soundbite,
  onClose,
  onUpdated,
}: {
  soundbite: SoundbiteResponse | null;
  onClose: () => void;
  onUpdated: (soundbite: SoundbiteResponse) => void;
}) {
  const isCreating = soundbite === null;
  const [isEditing, setIsEditing] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editDate, setEditDate] = useState("");
  const [editStreamNumber, setEditStreamNumber] = useState("");
  const [editSoundbiteNumber, setEditSoundbiteNumber] = useState("");
  const [editArtistIds, setEditArtistIds] = useState<string[]>([]);
  const [editFile, setEditFile] = useState<File | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const queryClient = useQueryClient();
  const isFormOpen = isCreating || isEditing;
  const streamPosition = parseStreamPosition(
    editStreamNumber,
    editSoundbiteNumber,
    "soundbite number",
  );

  const { data: allArtists } = useArtists({ per_page: 200 }, isFormOpen);

  const createMutation = useMutation({
    mutationFn: async () => {
      if (!streamPosition.ok) throw new Error(streamPosition.message);
      if (editFile === null) throw new Error("Choose an audio file.");
      const created = unwrap(
        await soundbitesApi.create({
          title: editTitle.trim(),
          soundbite_date: editDate,
          stream_number: streamPosition.stream_number,
          soundbite_number: streamPosition.position_number,
          artist_ids: editArtistIds,
        }),
      );
      unwrap(await soundbitesApi.uploadAudio(created.id, editFile));
    },
    onSuccess: () => {
      onClose();
    },
    onError: (error) => {
      setFormError(failureMessage("Failed to create soundbite", error));
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: soundbiteKeys.all() });
    },
  });

  const updateMutation = useMutation({
    mutationFn: async () => {
      if (!soundbite) throw new Error("No soundbite is selected.");
      if (!streamPosition.ok) throw new Error(streamPosition.message);
      const updated = unwrap(
        await soundbitesApi.update(soundbite.id, {
          title: editTitle.trim(),
          soundbite_date: editDate,
          stream_number: streamPosition.stream_number,
          soundbite_number: streamPosition.position_number,
          artist_ids: editArtistIds,
        }),
      );
      if (editFile === null) return updated;
      return unwrap(await soundbitesApi.uploadAudio(soundbite.id, editFile));
    },
    onSuccess: (updated) => {
      onUpdated(updated);
      setEditFile(null);
      setIsEditing(false);
      setFormError(null);
    },
    onError: (error) => {
      setFormError(failureMessage("Failed to update soundbite", error));
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: soundbiteKeys.all() });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      if (!soundbite) return;
      ensureOk(await soundbitesApi.delete(soundbite.id));
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: soundbiteKeys.all() });
      onClose();
    },
  });

  function startEditing() {
    if (!soundbite) return;
    setEditTitle(soundbite.title);
    setEditDate(soundbite.soundbite_date);
    setEditStreamNumber(soundbite.stream_number != null ? String(soundbite.stream_number) : "");
    setEditSoundbiteNumber(
      soundbite.soundbite_number != null ? String(soundbite.soundbite_number) : "",
    );
    setEditArtistIds(soundbite.artists.map((artist) => artist.id));
    setEditFile(null);
    setFormError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setEditFile(null);
    setIsEditing(false);
    setFormError(null);
  }

  function chooseFile(file: File | null) {
    setEditFile(file);
    if (file !== null && editTitle.trim() === "") setEditTitle(withoutExtension(file.name));
  }

  function toggleArtist(artistId: string) {
    setEditArtistIds((previous) =>
      previous.includes(artistId)
        ? previous.filter((id) => id !== artistId)
        : [...previous, artistId],
    );
  }

  if (isFormOpen) {
    const isPending = isCreating ? createMutation.isPending : updateMutation.isPending;
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{isCreating ? "New soundbite" : soundbite.title}</h3>
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
              disabled={
                isPending ||
                editTitle.trim() === "" ||
                editDate === "" ||
                !streamPosition.ok ||
                (isCreating && editFile === null)
              }
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
          <SingleAudioField
            label="Audio"
            currentName={soundbite?.audio ? assetLabel(soundbite.audio) : undefined}
            file={editFile}
            onChange={chooseFile}
          />
          <div className="form-field">
            <label className="form-label" htmlFor="soundbite-edit-title">
              Title
            </label>
            <input
              id="soundbite-edit-title"
              className="form-input"
              type="text"
              maxLength={MAX_TITLE_LENGTH}
              value={editTitle}
              onChange={(event) => {
                setEditTitle(event.target.value);
              }}
              required
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="soundbite-edit-date">
              Soundbite date
            </label>
            <input
              id="soundbite-edit-date"
              className="form-input"
              type="date"
              value={editDate}
              onChange={(event) => {
                setEditDate(event.target.value);
              }}
              required
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="soundbite-edit-stream">
              Stream number (blank if not streamed)
            </label>
            <input
              id="soundbite-edit-stream"
              className="form-input"
              type="number"
              min={1}
              value={editStreamNumber}
              onChange={(event) => {
                setEditStreamNumber(event.target.value);
              }}
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="soundbite-edit-number">
              Soundbite number (blank if not streamed)
            </label>
            <input
              id="soundbite-edit-number"
              className="form-input"
              type="number"
              min={1}
              value={editSoundbiteNumber}
              onChange={(event) => {
                setEditSoundbiteNumber(event.target.value);
              }}
            />
            {!streamPosition.ok && <p className="form-error">{streamPosition.message}</p>}
          </div>
          <ItemPicker
            label="Artists"
            allItems={allArtists?.items ?? []}
            selectedIds={editArtistIds}
            onToggle={toggleArtist}
            searchPlaceholder="Search artists…"
            getLabel={(artist) => artist.name}
          />
          {formError !== null && <p className="form-error">{formError}</p>}
        </div>
      </>
    );
  }

  const audioUrl = resolveAssetUrl(soundbite.audio);

  return (
    <>
      <div className="admin-panel-header">
        <h3 className="admin-panel-title">{soundbite.title}</h3>
        <div className="admin-tag-confirm">
          <button className="btn btn-secondary" onClick={startEditing}>
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
        error={
          deleteMutation.isError
            ? failureMessage("Failed to delete soundbite", deleteMutation.error)
            : null
        }
      />

      <div className="admin-panel-scroll">
        <div className="admin-detail-section">
          <span className="admin-detail-label">Date</span>
          <span className="admin-empty">{formatDate(soundbite.soundbite_date)}</span>
        </div>
        {soundbite.stream_number != null && soundbite.soundbite_number != null && (
          <div className="admin-detail-section">
            <span className="admin-detail-label">Stream</span>
            <span className="admin-empty">
              S{soundbite.stream_number} #{soundbite.soundbite_number}
            </span>
          </div>
        )}
        {soundbite.artists.length > 0 && (
          <div className="admin-detail-section">
            <span className="admin-detail-label">Artists</span>
            <div className="admin-pills">
              {soundbite.artists.map((artist) => (
                <span key={artist.id} className="admin-pill-display">
                  {artist.name}
                </span>
              ))}
            </div>
          </div>
        )}
        <div className="admin-detail-section">
          <span className="admin-detail-label">Audio</span>
          {soundbite.audio && audioUrl ? (
            <a href={audioUrl} target="_blank" rel="noreferrer" className="admin-link-url">
              {assetLabel(soundbite.audio)}
            </a>
          ) : (
            <span className="admin-empty">No audio uploaded.</span>
          )}
        </div>
      </div>
    </>
  );
}
