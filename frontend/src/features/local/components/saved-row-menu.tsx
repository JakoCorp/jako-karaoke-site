import { InfoIcon, TrashSimpleIcon } from "@phosphor-icons/react";
import { useNavigate } from "react-router";

import { RowContextMenu } from "@/components/row-context-menu";
import type { RowMenuItemDef } from "@/components/row-context-menu";

import { removeOffline } from "../lib/save-offline";

interface Props {
  performanceId: string;
  children: React.ReactNode;
  onPlay: () => void;
}

export function SavedRowMenu({ performanceId, children, onPlay }: Props) {
  const navigate = useNavigate();

  const items: RowMenuItemDef[] = [
    {
      icon: <InfoIcon size={14} />,
      label: "View details",
      onClick: () => {
        void navigate(`/performance/${performanceId}`);
      },
    },
    {
      icon: <TrashSimpleIcon size={14} />,
      label: "Delete",
      variant: "danger",
      confirm: { title: "Delete saved copy?", confirmLabel: "Delete" },
      onClick: () => {
        void removeOffline(performanceId);
      },
    },
  ];

  return (
    <RowContextMenu
      items={items}
      triggerClassName="local-perf-row"
      menuBtnAriaLabel="Saved performance options"
      onTriggerClick={onPlay}
    >
      {children}
    </RowContextMenu>
  );
}
