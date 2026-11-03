import { LinkSimpleIcon, PencilSimpleIcon, TrashSimpleIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { PlaylistResponse } from "@/api/playlists";
import { ConfirmDialog } from "@/components/confirm-dialog";

import { EditPlaylistDialog } from "./components/edit-playlist-dialog";
import { copyPlaylistLink } from "./lib/playlist-link";

export type PlaylistMenuAction = {
  key: string;
  icon: React.ReactNode;
  label: string;
  danger?: boolean;
  onSelect: () => void;
};

type Options = {
  playlist: PlaylistResponse;
  /** Present when the viewer can edit and delete the playlist. */
  onDelete?: () => void;
};

/**
 * Builds the actions of a playlist menu and the dialogs they open.
 *
 * The dialogs must be rendered outside the menu popup so they outlive it.
 */
export function usePlaylistMenu({ playlist, onDelete }: Options) {
  const [editOpen, setEditOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const actions: PlaylistMenuAction[] = [];
  if (playlist.is_public) {
    actions.push({
      key: "copy-link",
      icon: <LinkSimpleIcon size={14} />,
      label: "Copy link",
      onSelect: () => void copyPlaylistLink(playlist.id),
    });
  }
  if (onDelete) {
    actions.push(
      {
        key: "edit",
        icon: <PencilSimpleIcon size={14} />,
        label: "Edit",
        onSelect: () => setEditOpen(true),
      },
      {
        key: "delete",
        icon: <TrashSimpleIcon size={14} />,
        label: "Delete",
        danger: true,
        onSelect: () => setDeleteOpen(true),
      },
    );
  }

  const dialogs = onDelete && (
    <>
      <EditPlaylistDialog playlist={playlist} open={editOpen} onOpenChange={setEditOpen} />
      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        onConfirm={onDelete}
        title="Delete playlist?"
        description={`"${playlist.title}" will be permanently deleted.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </>
  );

  return { actions, dialogs };
}
