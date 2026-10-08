import { DownloadSimpleIcon, TrashSimpleIcon } from "@phosphor-icons/react";

import type { RowMenuItemDef } from "@/components/row-context-menu";
import { useOfflineStore } from "@/store/offline";

import { removeOffline, saveOffline } from "./lib/save-offline";

function reportFailure(error: unknown) {
  window.alert(error instanceof Error ? error.message : "Failed to save offline.");
}

interface OfflineMenuAction {
  isSaved: boolean;
  label: string;
  toggle: () => void;
}

/** Returns the save or remove action for a performance. */
export function useOfflineMenuAction(performanceId: string): OfflineMenuAction {
  const isSaved = useOfflineStore((state) => performanceId in state.entries);

  return {
    isSaved,
    label: isSaved ? "Remove from offline" : "Save offline",
    toggle: () => {
      const task = isSaved ? removeOffline(performanceId) : saveOffline(performanceId);
      task.catch(reportFailure);
    },
  };
}

/** Builds the "Save offline" row menu item. */
export function useOfflineMenuItem(performanceId: string): RowMenuItemDef {
  const action = useOfflineMenuAction(performanceId);
  return {
    icon: action.isSaved ? <TrashSimpleIcon size={14} /> : <DownloadSimpleIcon size={14} />,
    label: action.label,
    onClick: action.toggle,
  };
}
