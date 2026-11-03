import { type PlaylistResponse } from "@/api/playlists";
import { useUpdatePlaylist } from "@/hooks/api/playlists";
import { errorMessage } from "@/lib/error-message";

import { type PlaylistFormValues, PlaylistFormDialog } from "./playlist-form-dialog";

type Props = {
  playlist: PlaylistResponse;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function EditPlaylistDialog({ playlist, open, onOpenChange }: Props) {
  const updatePlaylist = useUpdatePlaylist();

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) updatePlaylist.reset();
    onOpenChange(nextOpen);
  }

  function handleSubmit(values: PlaylistFormValues) {
    updatePlaylist.mutate(
      {
        id: playlist.id,
        body: {
          title: values.title,
          description: values.description || null,
          is_public: values.isPublic,
        },
      },
      { onSuccess: () => handleOpenChange(false) },
    );
  }

  return (
    <PlaylistFormDialog
      open={open}
      onOpenChange={handleOpenChange}
      heading="Edit playlist"
      initialValues={{
        title: playlist.title,
        description: playlist.description ?? "",
        isPublic: playlist.is_public,
      }}
      submitLabel="Save"
      pendingLabel="Saving…"
      isPending={updatePlaylist.isPending}
      error={
        updatePlaylist.isError
          ? errorMessage(updatePlaylist.error, "Failed to update playlist.")
          : null
      }
      onSubmit={handleSubmit}
    />
  );
}
