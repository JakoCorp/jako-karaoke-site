/** Resolved embed info for an external video URL. */
export interface VideoEmbedInfo {
  /** Iframe src URL. Null when the platform cannot be embedded. */
  embedUrl: string | null;
  /** Iframe src URL with autoplay enabled. Null when the platform cannot be embedded. */
  autoplayEmbedUrl: string | null;
  /** Static preview image URL. Null when unavailable without an API call. */
  thumbnailUrl: string | null;
  /** Human-readable platform label, e.g. "YouTube" or "Twitch". */
  platform: string | null;
}

/**
 * Derives embed, autoplay embed, and thumbnail info from an external video URL.
 * Add new platforms by inserting a case before the final fallback return.
 */
export function getVideoEmbedInfo(externalUrl: string): VideoEmbedInfo {
  // YouTube: standard watch URL and short URL
  const ytMatch = /(?:youtube\.com\/watch\?v=|youtu\.be\/)([A-Za-z0-9_-]{11})/.exec(externalUrl);
  if (ytMatch) {
    const id = ytMatch[1]!;
    return {
      embedUrl: `https://www.youtube.com/embed/${id}`,
      autoplayEmbedUrl: `https://www.youtube.com/embed/${id}?autoplay=1`,
      thumbnailUrl: `https://img.youtube.com/vi/${id}/mqdefault.jpg`,
      platform: "YouTube",
    };
  }

  // Twitch clip: twitch.tv/{channel}/clip/{slug} or clips.twitch.tv/{slug}
  const twitchClip =
    /twitch\.tv\/\w+\/clip\/([A-Za-z0-9_-]+)/.exec(externalUrl) ??
    /clips\.twitch\.tv\/([A-Za-z0-9_-]+)/.exec(externalUrl);
  if (twitchClip) {
    const parent = window.location.hostname;
    const base = `https://clips.twitch.tv/embed?clip=${twitchClip[1]!}&parent=${parent}`;
    return {
      embedUrl: base,
      autoplayEmbedUrl: `${base}&autoplay=true`,
      thumbnailUrl: null,
      platform: "Twitch",
    };
  }

  // Twitch VOD: twitch.tv/videos/{id}
  const twitchVod = /twitch\.tv\/videos\/(\d+)/.exec(externalUrl);
  if (twitchVod) {
    const parent = window.location.hostname;
    const base = `https://player.twitch.tv/?video=${twitchVod[1]!}&parent=${parent}`;
    return {
      embedUrl: base,
      autoplayEmbedUrl: `${base}&autoplay=true`,
      thumbnailUrl: null,
      platform: "Twitch",
    };
  }

  return { embedUrl: null, autoplayEmbedUrl: null, thumbnailUrl: null, platform: null };
}

/**
 * Extracts the 11-character video ID from a YouTube watch or short URL.
 * Returns null for non-YouTube URLs.
 */
export function extractYouTubeVideoId(url: string): string | null {
  return /(?:youtube\.com\/watch\?v=|youtu\.be\/)([A-Za-z0-9_-]{11})/.exec(url)?.[1] ?? null;
}
