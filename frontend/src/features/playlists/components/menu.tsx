import { ContextMenu, Menu } from "@base-ui/react";
import { DotsThreeVerticalIcon, TrashSimpleIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { ConfirmDialog } from "@/components/confirm-dialog";

function MenuItems({ onRequestDelete }: { onRequestDelete: () => void }) {
  return (
    <Menu.Item className="card-menu-item card-menu-item--danger" onClick={onRequestDelete}>
      <TrashSimpleIcon size={14} />
      Delete
    </Menu.Item>
  );
}

function ContextMenuItems({ onRequestDelete }: { onRequestDelete: () => void }) {
  return (
    <ContextMenu.Item className="card-menu-item card-menu-item--danger" onClick={onRequestDelete}>
      <TrashSimpleIcon size={14} />
      Delete
    </ContextMenu.Item>
  );
}

type CardMenuProps = {
  playlistTitle: string;
  onDelete: () => void;
  children: React.ReactNode;
};

export function PlaylistCardMenu({ playlistTitle, onDelete, children }: CardMenuProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);

  return (
    <>
      <ContextMenu.Root>
        <ContextMenu.Trigger className="playlist-card-wrap">
          {children}
          <Menu.Root>
            <Menu.Trigger className="playlist-card-menu-btn" aria-label="Playlist options">
              <DotsThreeVerticalIcon size={16} />
            </Menu.Trigger>
            <Menu.Portal>
              <Menu.Positioner>
                <Menu.Popup className="card-menu-popup">
                  <MenuItems onRequestDelete={() => setConfirmOpen(true)} />
                </Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.Root>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Positioner>
            <ContextMenu.Popup className="card-menu-popup">
              <ContextMenuItems onRequestDelete={() => setConfirmOpen(true)} />
            </ContextMenu.Popup>
          </ContextMenu.Positioner>
        </ContextMenu.Portal>
      </ContextMenu.Root>
      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        onConfirm={onDelete}
        title="Delete playlist?"
        description={`"${playlistTitle}" will be permanently deleted.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </>
  );
}

type DetailMenuProps = {
  playlistTitle: string;
  onDelete: () => void;
};

export function PlaylistDetailMenu({ playlistTitle, onDelete }: DetailMenuProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);

  return (
    <>
      <Menu.Root>
        <Menu.Trigger className="playlist-detail-menu-btn" aria-label="Playlist options">
          <DotsThreeVerticalIcon size={18} />
        </Menu.Trigger>
        <Menu.Portal>
          <Menu.Positioner>
            <Menu.Popup className="card-menu-popup">
              <MenuItems onRequestDelete={() => setConfirmOpen(true)} />
            </Menu.Popup>
          </Menu.Positioner>
        </Menu.Portal>
      </Menu.Root>
      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        onConfirm={onDelete}
        title="Delete playlist?"
        description={`"${playlistTitle}" will be permanently deleted.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </>
  );
}
