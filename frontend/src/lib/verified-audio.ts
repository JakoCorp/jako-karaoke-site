import type { AudioInfo, PerformanceResponse } from "@/api/performances";
import type { AssetInfo } from "@/api/types";

/** A hosted audio asset that carries the SHA-256 needed to verify a download. */
export type VerifiableAudio = AssetInfo & { storage_url: string; hash: string };

/** Returns whether an asset is hosted and has a hash, so a download of it can be verified. */
export function isVerifiable(asset: AssetInfo): asset is VerifiableAudio {
  return !!asset.storage_url && !!asset.hash;
}

/** Returns the primary hosted audio asset of a performance, if one can be downloaded. */
export function findDownloadableAudio(
  detail: PerformanceResponse,
): (AudioInfo & VerifiableAudio) | undefined {
  return detail.audio.find(
    (audio): audio is AudioInfo & VerifiableAudio =>
      audio.kind === "primary" && isVerifiable(audio),
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
