import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";

/** Returns the app path of a playlist's detail page. */
export function playlistPath(id: string): string {
  return `/playlist/${id}`;
}

/** Copies the full link to a playlist to the clipboard and reports the outcome as a toast. */
export async function copyPlaylistLink(id: string): Promise<void> {
  const link = new URL(playlistPath(id), window.location.origin).href;
  try {
    await navigator.clipboard.writeText(link);
    toast.success("Link copied");
  } catch (error) {
    toast.error("Could not copy link", {
      description: errorMessage(error, "Something went wrong."),
    });
  }
}
