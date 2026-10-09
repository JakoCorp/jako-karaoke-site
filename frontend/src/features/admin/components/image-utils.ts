export interface AdminImage {
  asset_id: string;
  title?: string | null;
  storage_url?: string | null;
  external_url?: string | null;
  kind: string;
}

export function imageLabel(image: AdminImage): string {
  return image.title ?? image.storage_url?.split("/").pop() ?? image.external_url ?? "Untitled";
}
