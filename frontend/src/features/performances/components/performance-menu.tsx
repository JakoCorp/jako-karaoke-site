import { ContextMenu, Menu } from "@base-ui/react";
import {
  CaretRightIcon,
  DotsThreeVerticalIcon,
  InfoIcon,
  MusicNotesPlusIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "react-router";

import { PlaylistPickerContent } from "@/features/playlists";

interface RowMenuProps {
  performanceId: string;
  children: React.ReactNode;
}

interface DetailMenuProps {
  performanceId: string;
}

interface MenuItemsProps {
  performanceId: string;
  onViewDetails?: () => void;
}

function AddToPlaylistSubmenu({ performanceId }: { performanceId: string }) {
  return (
    <Menu.SubmenuRoot>
      <Menu.SubmenuTrigger className="card-menu-item">
        <MusicNotesPlusIcon size={14} />
        Add to playlist
        <CaretRightIcon size={12} style={{ marginLeft: "auto" }} />
      </Menu.SubmenuTrigger>
      <Menu.Portal>
        <Menu.Positioner side="right" align="start" sideOffset={4}>
          <Menu.Popup className="playlist-picker-popup">
            <PlaylistPickerContent performanceId={performanceId} />
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.SubmenuRoot>
  );
}

function MenuItems({ performanceId, onViewDetails }: MenuItemsProps) {
  return (
    <>
      {onViewDetails && (
        <Menu.Item className="card-menu-item" onClick={onViewDetails}>
          <InfoIcon size={14} />
          View details
        </Menu.Item>
      )}
      <AddToPlaylistSubmenu performanceId={performanceId} />
    </>
  );
}

function ContextMenuItems({ performanceId, onViewDetails }: MenuItemsProps) {
  return (
    <>
      {onViewDetails && (
        <ContextMenu.Item className="card-menu-item" onClick={onViewDetails}>
          <InfoIcon size={14} />
          View details
        </ContextMenu.Item>
      )}
      <AddToPlaylistSubmenu performanceId={performanceId} />
    </>
  );
}

export function PerformanceRowMenu({ performanceId, children }: RowMenuProps) {
  const navigate = useNavigate();

  function handleViewDetails() {
    void navigate(`/performance/${performanceId}`);
  }

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger className="perf-row">
        {children}
        <Menu.Root>
          <Menu.Trigger className="perf-row-menu-btn" aria-label="Performance options">
            <DotsThreeVerticalIcon size={16} />
          </Menu.Trigger>
          <Menu.Portal>
            <Menu.Positioner>
              <Menu.Popup className="card-menu-popup">
                <MenuItems performanceId={performanceId} onViewDetails={handleViewDetails} />
              </Menu.Popup>
            </Menu.Positioner>
          </Menu.Portal>
        </Menu.Root>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Positioner>
          <ContextMenu.Popup className="card-menu-popup">
            <ContextMenuItems performanceId={performanceId} onViewDetails={handleViewDetails} />
          </ContextMenu.Popup>
        </ContextMenu.Positioner>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}

export function PerformanceDetailMenu({ performanceId }: DetailMenuProps) {
  return (
    <Menu.Root>
      <Menu.Trigger className="perf-detail-action-btn" aria-label="More options">
        <DotsThreeVerticalIcon size={20} />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Positioner>
          <Menu.Popup className="card-menu-popup">
            <MenuItems performanceId={performanceId} />
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.Root>
  );
}
