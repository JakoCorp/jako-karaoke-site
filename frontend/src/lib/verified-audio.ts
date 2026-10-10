import type { AudioInfo, PerformanceResponse } from "@/api/performances";

/** A hosted audio asset that carries the SHA-256 needed to verify a download. */
export type VerifiableAudio = AudioInfo & { storage_url: string; hash: string };

/** Returns the primary hosted audio asset of a performance, if one can be downloaded. */
export function findDownloadableAudio(detail: PerformanceResponse): VerifiableAudio | undefined {
  return detail.audio.find(
    (audio): audio is VerifiableAudio =>
      audio.kind === "primary" && !!audio.storage_url && !!audio.hash,
  );
}

async function sha256Hex(buffer: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", buffer);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/**
 * Downloads a hosted audio asset and verifies it against the server hash.
 *
 * Throws when the request fails or the downloaded bytes do not match the hash.
 */
export async function fetchVerifiedAudio(audio: VerifiableAudio): Promise<ArrayBuffer> {
  const response = await fetch(audio.storage_url);
  if (!response.ok) throw new Error("Failed to download audio.");
  const buffer = await response.arrayBuffer();

  if ((await sha256Hex(buffer)) !== audio.hash) {
    throw new Error("Downloaded audio did not match the server copy.");
  }
  return buffer;
}
