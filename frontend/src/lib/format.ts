export function formatDuration(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = Math.floor(totalSeconds % 60);
  return `${String(minutes)}:${String(seconds).padStart(2, "0")}`;
}

export function formatDate(dateString: string): string {
  // Force local timezone interpretation.
  const date = dateString.includes("T") ? new Date(dateString) : new Date(dateString + "T00:00:00");
  return new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  }).format(date);
}

function padTwo(n: number): string {
  return String(n).padStart(2, "0");
}

export function isoToDatetimeLocal(isoString: string): string {
  const date = new Date(isoString);
  return `${String(date.getFullYear())}-${padTwo(date.getMonth() + 1)}-${padTwo(date.getDate())}T${padTwo(date.getHours())}:${padTwo(date.getMinutes())}`;
}

export function formatStreamTime(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = Math.floor(totalSeconds % 60);
  return `${String(hours)}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

/**
 * Parses a stream time string into total seconds.
 *
 * Accepts H:MM:SS, H.MM.SS, MM:SS, MM.SS, or a bare integer (seconds).
 * Returns `null` for empty or unparseable input.
 */
export function parseStreamTime(input: string): number | null {
  const trimmed = input.trim();
  if (trimmed === "") return null;

  const normalized = trimmed.replace(/\./g, ":");
  const parts = normalized.split(":").map(Number);

  if (parts.some((n) => isNaN(n))) return null;

  const a = parts[0] ?? 0;
  const b = parts[1] ?? 0;
  const c = parts[2] ?? 0;

  if (parts.length === 1) return a;
  if (parts.length === 2) return a * 60 + b;
  if (parts.length === 3) return a * 3600 + b * 60 + c;
  return null;
}

export function formatRelativeDate(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${String(diffDays)} days ago`;
  return formatDate(isoString);
}
