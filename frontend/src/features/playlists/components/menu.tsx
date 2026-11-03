import { ContextMenu, Menu } from "@base-ui/react";
import { DotsThreeVerticalIcon } from "@phosphor-icons/react";

import type { PlaylistResponse } from "@/api/playlists";

import { type PlaylistMenuAction, usePlaylistMenu } from "../use-playlist-menu";

function itemClassName(action: PlaylistMenuAction) {
  return action.danger ? "card-menu-item card-menu-item--danger" : "card-menu-item";
}

function MenuItems({ actions }: { actions: PlaylistMenuAction[] }) {
  return (
    <>
      {actions.map((action) => (
        <Menu.Item key={action.key} className={itemClassName(action)} onClick={action.onSelect}>
          {action.icon}
          {action.label}
        </Menu.Item>
      ))}
    </>
  );
}

function ContextMenuItems({ actions }: { actions: PlaylistMenuAction[] }) {
  return (
    <>
      {actions.map((action) => (
        <ContextMenu.Item
          key={action.key}
          className={itemClassName(action)}
          onClick={action.onSelect}
        >
          {action.icon}
          {action.label}
        </ContextMenu.Item>
      ))}
    </>
  );
}

type CardMenuProps = {
  playlist: PlaylistResponse;
  onDelete: () => void;
  children: React.ReactNode;
};

export function PlaylistCardMenu({ playlist, onDelete, children }: CardMenuProps) {
  const { actions, dialogs } = usePlaylistMenu({ playlist, onDelete });

  return (
    <>
      <ContextMenu.Root>
        <ContextMenu.Trigger className="playlist-card-wrap">
          {children}
          <Menu.Root>
            <Menu.Trigger className="playlist-card-menu-btn" aria-label="Playlist options">
              <DotsThreeVerticalIcon size={18} />
            </Menu.Trigger>
            <Menu.Portal>
              <Menu.Positioner>
                <Menu.Popup className="card-menu-popup">
                  <MenuItems actions={actions} />
                </Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.Root>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Positioner>
            <ContextMenu.Popup className="card-menu-popup">
              <ContextMenuItems actions={actions} />
            </ContextMenu.Popup>
          </ContextMenu.Positioner>
        </ContextMenu.Portal>
      </ContextMenu.Root>
      {dialogs}
    </>
  );
}

type DetailMenuProps = {
  playlist: PlaylistResponse;
  onDelete?: () => void;
};

export function PlaylistDetailMenu({ playlist, onDelete }: DetailMenuProps) {
  const { actions, dialogs } = usePlaylistMenu({ playlist, onDelete });

  if (actions.length === 0) return null;

  return (
    <>
      <Menu.Root>
        <Menu.Trigger className="playlist-detail-menu-btn" aria-label="Playlist options">
          <DotsThreeVerticalIcon size={20} />
        </Menu.Trigger>
        <Menu.Portal>
          <Menu.Positioner>
            <Menu.Popup className="card-menu-popup">
              <MenuItems actions={actions} />
            </Menu.Popup>
          </Menu.Positioner>
        </Menu.Portal>
      </Menu.Root>
      {dialogs}
    </>
  );
}
