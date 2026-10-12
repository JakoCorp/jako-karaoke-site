import { Menu } from "@base-ui/react";
import {
  CaretRightIcon,
  DotsThreeVerticalIcon,
  DownloadSimpleIcon,
  FileArrowDownIcon,
  InfoIcon,
  MusicNotesPlusIcon,
  TrashSimpleIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "react-router";

import { RowContextMenu } from "@/components/row-context-menu";
import type { RowMenuItemDef } from "@/components/row-context-menu";
import { useDownloadMenuAction, useDownloadMenuItem } from "@/features/download";
import { useOfflineMenuAction, useOfflineMenuItem } from "@/features/local";
import { PlaylistPickerContent } from "@/features/playlists";

interface RowMenuProps {
  performanceId: string;
  children: React.ReactNode;
  onPlay?: () => void;
}

interface DetailMenuProps {
  performanceId: string;
}

export function PerformanceRowMenu({ performanceId, children, onPlay }: RowMenuProps) {
  const navigate = useNavigate();
  const downloadItem = useDownloadMenuItem(performanceId);
  const offlineItem = useOfflineMenuItem(performanceId);
  const items: RowMenuItemDef[] = [
    {
      icon: <InfoIcon size={14} />,
      label: "View details",
      onClick: () => {
        void navigate(`/performance/${performanceId}`);
      },
    },
    downloadItem,
    offlineItem,
    {
      type: "submenu",
      icon: <MusicNotesPlusIcon size={14} />,
      label: "Add to playlist",
      content: <PlaylistPickerContent performanceId={performanceId} />,
    },
  ];

  return (
    <RowContextMenu
      items={items}
      triggerClassName="perf-row"
      menuBtnAriaLabel="Performance options"
      onTriggerClick={onPlay}
    >
      {children}
    </RowContextMenu>
  );
}

export function PerformanceDetailMenu({ performanceId }: DetailMenuProps) {
  const downloadAction = useDownloadMenuAction(performanceId);
  const offlineAction = useOfflineMenuAction(performanceId);
  return (
    <Menu.Root>
      <Menu.Trigger className="perf-detail-action-btn" aria-label="More options">
        <DotsThreeVerticalIcon size={20} />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Positioner>
          <Menu.Popup className="card-menu-popup">
            <Menu.Item className="card-menu-item" onClick={downloadAction.download}>
              <FileArrowDownIcon size={14} />
              {downloadAction.label}
            </Menu.Item>
            <Menu.Item className="card-menu-item" onClick={offlineAction.toggle}>
              {offlineAction.isSaved ? (
                <TrashSimpleIcon size={14} />
              ) : (
                <DownloadSimpleIcon size={14} />
              )}
              {offlineAction.label}
            </Menu.Item>
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
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.Root>
  );
}
