import { queryOptions } from "@tanstack/react-query";
import { useEffect } from "react";

import { performancesApi } from "@/api/performances";
import { performanceKeys } from "@/hooks/api/performances";
import { queryClient } from "@/lib/query-client";
import { selectCurrent, usePlayerStore } from "@/store/player";

function performanceDetailOptions(id: string) {
  return queryOptions({
    queryKey: performanceKeys.detail(id),
    queryFn: async () => {
      const { data, error } = await performancesApi.get(id);
      if (error) throw error;
      return data;
    },
  });
}

/** Resolves the audio URL and thumbnail for the active queue track. */
export function useCurrentTrackResolver(): void {
  const current = usePlayerStore(selectCurrent);
  const nextPerf = usePlayerStore((s) =>
    s.queueIndex >= 0 ? (s.queue[s.queueIndex + 1] ?? null) : null,
  );
  const setCurrentAudioUrl = usePlayerStore((s) => s.setCurrentAudioUrl);
  const setCurrentThumbnailUrl = usePlayerStore((s) => s.setCurrentThumbnailUrl);

  const currentId = current?.id ?? null;
  const nextId = nextPerf?.id ?? null;

  useEffect(() => {
    let cancelled = false;

    if (currentId !== null) {
      void (async () => {
        try {
          const detail = await queryClient.ensureQueryData(performanceDetailOptions(currentId));
          if (cancelled) return;
          const audioUrl = detail.audio.find((a) => a.kind === "primary")?.public_url ?? null;
          const thumbnailUrl =
            detail.songs[0]?.images.find((img) => img.kind === "cover_art")?.public_url ?? null;
          setCurrentAudioUrl(audioUrl);
          setCurrentThumbnailUrl(thumbnailUrl);
        } catch {
          // Fetch failed, URLs remain null
        }
      })();
    } else {
      setCurrentAudioUrl(null);
      setCurrentThumbnailUrl(null);
    }

    return () => {
      cancelled = true;
    };
  }, [currentId, setCurrentAudioUrl, setCurrentThumbnailUrl]);

  useEffect(() => {
    if (nextId === null) return;
    void queryClient.ensureQueryData(performanceDetailOptions(nextId)).catch(() => {
      // Prefetch failure is non-fatal
    });
  }, [nextId]);
}
