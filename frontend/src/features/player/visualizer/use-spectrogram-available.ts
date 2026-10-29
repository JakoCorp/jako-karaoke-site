import { extractYouTubeVideoId } from "@/lib/video-embed";
import { usePlayerStore } from "@/store/player";

/** Whether the current track plays through an audio element that can be analysed. */
export function useSpectrogramAvailable(): boolean {
  const currentAudioUrl = usePlayerStore((s) => s.currentAudioUrl);
  return currentAudioUrl !== null && extractYouTubeVideoId(currentAudioUrl) === null;
}
