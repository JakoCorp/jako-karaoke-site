import { type PlaylistResponse } from "@/api/playlists";
import { useCreatePlaylist } from "@/hooks/api/playlists";
import { errorMessage } from "@/lib/error-message";

import { type PlaylistFormValues, PlaylistFormDialog } from "./playlist-form-dialog";

const EMPTY_VALUES: PlaylistFormValues = { title: "", description: "", isPublic: false };

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (playlist: PlaylistResponse) => void;
};

export function CreatePlaylistDialog({ open, onOpenChange, onCreated }: Props) {
  const createPlaylist = useCreatePlaylist();

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) createPlaylist.reset();
    onOpenChange(nextOpen);
  }

  function handleSubmit(values: PlaylistFormValues) {
    createPlaylist.mutate(
      {
        title: values.title,
        description: values.description || null,
        kind: "user",
        is_public: values.isPublic,
      },
      {
        onSuccess: (playlist) => {
          onCreated(playlist);
          handleOpenChange(false);
        },
      },
    );
  }

  return (
    <PlaylistFormDialog
      open={open}
      onOpenChange={handleOpenChange}
      heading="New playlist"
      initialValues={EMPTY_VALUES}
      submitLabel="Create"
      pendingLabel="Creating…"
      isPending={createPlaylist.isPending}
      error={
        createPlaylist.isError
          ? errorMessage(createPlaylist.error, "Failed to create playlist.")
          : null
      }
      onSubmit={handleSubmit}
    />
  );
}
