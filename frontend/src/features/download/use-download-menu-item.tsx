import { FileArrowDownIcon } from "@phosphor-icons/react";
import { useQueryClient } from "@tanstack/react-query";

import { DEFAULT_DOWNLOAD_SETTINGS } from "@/api/settings";
import type { DownloadSettings } from "@/api/settings";
import type { RowMenuItemDef } from "@/components/row-context-menu";
import { userSettingsQueryOptions } from "@/hooks/api/settings";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

import { downloadPerformance } from "./lib/download-performance";

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
      toast.warning("Using default download settings", {
        description: "Your saved download settings could not be loaded.",
      });
      return DEFAULT_DOWNLOAD_SETTINGS;
    }
  }

  return {
    label: "Download",
    download: () => {
      void toast.task(
        resolveSettings().then((settings) => downloadPerformance(performanceId, settings)),
        {
          loading: "Preparing download…",
          success: (result) =>
            result.tagsFailed
              ? {
                  type: "warning",
                  title: "Downloaded without tags",
                  description: result.filename,
                }
              : { title: "Downloaded", description: result.filename },
          error: (error) => ({
            title: "Download failed",
            description: errorMessage(error, "Something went wrong while downloading."),
          }),
        },
      );
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
