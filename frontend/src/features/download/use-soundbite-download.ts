import type { SoundbiteResponse } from "@/api/soundbites";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";

import { downloadSoundbite, isSoundbiteDownloadable } from "./lib/download-soundbite";

interface SoundbiteDownloadAction {
  label: string;

  /** False when the soundbite has no hosted audio to download. */
  isAvailable: boolean;
  download: () => void;
}

/** Returns the download action for a soundbite. */
export function useSoundbiteDownloadAction(soundbite: SoundbiteResponse): SoundbiteDownloadAction {
  return {
    label: "Download",
    isAvailable: isSoundbiteDownloadable(soundbite),
    download: () => {
      void toast.task(downloadSoundbite(soundbite), {
        loading: "Preparing download…",
        success: (filename) => ({ title: "Downloaded", description: filename }),
        error: (error) => ({
          title: "Download failed",
          description: errorMessage(error, "Something went wrong while downloading."),
        }),
      });
    },
  };
}
