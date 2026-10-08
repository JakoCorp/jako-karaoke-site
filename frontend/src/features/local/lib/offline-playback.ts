import { useOfflineStore } from "@/store/offline";

import { getAudioBlob } from "./offline-db";

let activeObjectUrl: string | null = null;

/** Revokes the object URL handed out by the previous `openOfflineAudio` call. */
export function releaseOfflineAudio(): void {
  if (activeObjectUrl) {
    URL.revokeObjectURL(activeObjectUrl);
    activeObjectUrl = null;
  }
}

/**
 * Returns a playable `blob:` URL for a saved performance, or `null` when the saved copy
 * should not be used.
 *
 * `serverHash` is the hash the server currently reports for the audio. Pass `undefined`
 * when the server is unreachable to use the saved copy unconditionally.
 */
export async function openOfflineAudio(
  performanceId: string,
  serverHash: string | null | undefined,
): Promise<string | null> {
  releaseOfflineAudio();
  const entry = useOfflineStore.getState().entries[performanceId];
  if (!entry) return null;
  if (serverHash !== undefined && serverHash !== entry.hash) return null;

  const blob = await getAudioBlob(performanceId);
  if (!blob) return null;

  activeObjectUrl = URL.createObjectURL(blob);
  return activeObjectUrl;
}
