import { FileArrowDownIcon } from "@phosphor-icons/react";
import { useQueryClient } from "@tanstack/react-query";

import { DEFAULT_DOWNLOAD_SETTINGS } from "@/api/settings";
import type { DownloadSettings } from "@/api/settings";
import type { RowMenuItemDef } from "@/components/row-context-menu";
import { userSettingsQueryOptions } from "@/hooks/api/settings";
import { useAuthStore } from "@/store/auth";

import { downloadPerformance } from "./lib/download-performance";

function reportFailure(error: unknown) {
  window.alert(error instanceof Error ? error.message : "Failed to download.");
}

interface DownloadMenuAction {
  label: string;
  download: () => void;
}

/** Returns the download action for a performance, using the signed in user's preferences. */
export function useDownloadMenuAction(performanceId: string): DownloadMenuAction {
  const queryClient = useQueryClient();
  const isSignedIn = useAuthStore((state) => state.user !== null);

  async function resolveSettings(): Promise<DownloadSettings> {
    if (!isSignedIn) return DEFAULT_DOWNLOAD_SETTINGS;
    try {
      const settings = await queryClient.ensureQueryData(userSettingsQueryOptions());
      return settings.download;
    } catch {
      return DEFAULT_DOWNLOAD_SETTINGS;
    }
  }

  return {
    label: "Download",
    download: () => {
      void resolveSettings()
        .then((settings) => downloadPerformance(performanceId, settings))
        .catch(reportFailure);
    },
  };
}

/** Builds the "Download" row menu item. */
export function useDownloadMenuItem(performanceId: string): RowMenuItemDef {
  const action = useDownloadMenuAction(performanceId);
  return {
    icon: <FileArrowDownIcon size={14} />,
    label: action.label,
    onClick: action.download,
  };
}
