import type { SoundbiteResponse } from "@/api/soundbites";
import { fetchVerifiedAudio, isVerifiable } from "@/lib/verified-audio";

import { sanitizeFilename } from "./filename-template";
import { extensionOf, saveBlob } from "./save-file";

const FALLBACK_FILENAME = "soundbite";

/** Returns whether a soundbite has hosted audio with a hash, so its download can be verified. */
export function isSoundbiteDownloadable(soundbite: SoundbiteResponse): boolean {
  return !!soundbite.audio && isVerifiable(soundbite.audio);
}

/**
 * Downloads the audio of a soundbite to disk as `<title>.<extension>` and returns the filename.
 *
 * The audio is verified against the server hash and saved as it is, without tags. Throws when
 * there is no downloadable audio or the download fails.
 */
export async function downloadSoundbite(soundbite: SoundbiteResponse): Promise<string> {
  const audio = soundbite.audio;
  if (!audio || !isVerifiable(audio)) {
    throw new Error("This soundbite has no downloadable audio.");
  }

  const bytes = await fetchVerifiedAudio(audio);
  const filename = sanitizeFilename(
    soundbite.title,
    FALLBACK_FILENAME,
    extensionOf(audio.storage_url),
  );
  saveBlob(new Blob([bytes]), filename);
  return filename;
}
