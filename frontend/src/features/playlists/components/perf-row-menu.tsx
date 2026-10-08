import { InfoIcon, MusicNotesPlusIcon, TrashSimpleIcon } from "@phosphor-icons/react";
import { useNavigate } from "react-router";

import { RowContextMenu } from "@/components/row-context-menu";
import type { RowMenuItemDef } from "@/components/row-context-menu";
import { useOfflineMenuItem } from "@/features/local";

import { PlaylistPickerContent } from "./picker-content";

interface Props {
  performanceId: string;
  isOwner: boolean;
  children: React.ReactNode;
  onPlay: () => void;
  onRemove: () => void;
}

export function PlaylistPerfRowMenu({ performanceId, isOwner, children, onPlay, onRemove }: Props) {
  const navigate = useNavigate();
  const offlineItem = useOfflineMenuItem(performanceId);

  const items: RowMenuItemDef[] = [
    {
      icon: <InfoIcon size={14} />,
      label: "View details",
      onClick: () => {
        void navigate(`/performance/${performanceId}`);
      },
    },
    offlineItem,
    {
      type: "submenu",
      icon: <MusicNotesPlusIcon size={14} />,
      label: "Add to playlist",
      content: <PlaylistPickerContent performanceId={performanceId} />,
    },
    ...(isOwner
      ? [
          {
            icon: <TrashSimpleIcon size={14} />,
            label: "Remove from playlist",
            onClick: onRemove,
            variant: "danger" as const,
            confirm: { title: "Remove from playlist?", confirmLabel: "Remove" },
          } satisfies RowMenuItemDef,
        ]
      : []),
  ];

  return (
    <RowContextMenu
      items={items}
      triggerClassName="playlist-perf-row"
      menuBtnAriaLabel="Performance options"
      onTriggerClick={onPlay}
    >
      {children}
    </RowContextMenu>
  );
}
