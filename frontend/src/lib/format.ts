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
