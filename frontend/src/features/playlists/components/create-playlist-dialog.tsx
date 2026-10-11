import { Dialog } from "@base-ui/react";
import { useState } from "react";

import { type PlaylistResponse } from "@/api/playlists";
import { useCreatePlaylist } from "@/hooks/api/playlists";
import { errorMessage } from "@/lib/error-message";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (playlist: PlaylistResponse) => void;
};

export function CreatePlaylistDialog({ open, onOpenChange, onCreated }: Props) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [isPublic, setIsPublic] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createPlaylist = useCreatePlaylist();
  const isPending = createPlaylist.isPending;

  function resetForm() {
    setTitle("");
    setDescription("");
    setIsPublic(false);
    setError(null);
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    createPlaylist.mutate(
      {
        title: title.trim(),
        description: description.trim() || null,
        kind: "user",
        is_public: isPublic,
      },
      {
        onSuccess: (playlist) => {
          onCreated(playlist);
          resetForm();
        },
        onError: (mutationError) => {
          setError(errorMessage(mutationError, "Failed to create playlist."));
        },
      },
    );
  }

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !isPending) {
          resetForm();
          onOpenChange(false);
        } else if (nextOpen) {
          onOpenChange(true);
        }
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Title className="dialog-title">New playlist</Dialog.Title>
          <form onSubmit={handleSubmit} className="dialog-form">
            <div className="form-field">
              <label className="form-label" htmlFor="cp-title">
                Title
              </label>
              <input
                id="cp-title"
                className="form-input"
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                autoComplete="off"
              />
            </div>
            <div className="form-field">
              <label className="form-label" htmlFor="cp-desc">
                Description
              </label>
              <textarea
                id="cp-desc"
                className="form-input"
                rows={3}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>
            <label className="form-checkbox">
              <input
                type="checkbox"
                checked={isPublic}
                onChange={(e) => setIsPublic(e.target.checked)}
              />
              Make public
            </label>
            {error !== null && <p className="form-error">{error}</p>}
            <div className="dialog-actions">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  resetForm();
                  onOpenChange(false);
                }}
                disabled={isPending}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={!title.trim() || isPending}
              >
                {isPending ? "Creating…" : "Create"}
              </button>
            </div>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
