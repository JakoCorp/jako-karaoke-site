const OBJECT_URL_LIFETIME_MS = 60_000;

/** Returns the lowercase file extension of a URL path, or an empty string when it has none. */
export function extensionOf(url: string): string {
  const name = new URL(url).pathname.split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot === -1 ? "" : name.slice(dot + 1).toLowerCase();
}

/** Saves a blob to disk under the given filename through a temporary download link. */
export function saveBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => {
    URL.revokeObjectURL(url);
  }, OBJECT_URL_LIFETIME_MS);
}
