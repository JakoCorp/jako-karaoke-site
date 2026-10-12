import { DownloadSimpleIcon, TrashSimpleIcon } from "@phosphor-icons/react";

import type { RowMenuItemDef } from "@/components/row-context-menu";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { useOfflineStore } from "@/store/offline";

import { reportOfflineChange } from "./lib/report-offline-change";
import { removeOffline, saveOffline } from "./lib/save-offline";

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
      if (isSaved) {
        void reportOfflineChange(removeOffline(performanceId), {
          success: "Removed from offline",
          failure: "Could not remove from offline",
        });
        return;
      }
      void toast.task(saveOffline(performanceId), {
        loading: "Saving offline…",
        success: () => ({ title: "Saved offline" }),
        error: (error) => ({
          title: "Could not save offline",
          description: errorMessage(error, "Something went wrong."),
        }),
      });
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
