export interface AdminAsset {
  asset_id: string;
  title?: string | null;
  storage_url?: string | null;
  external_url?: string | null;
  kind: string;
}

export function assetLabel(
  asset: Pick<AdminAsset, "title" | "storage_url" | "external_url">,
): string {
  return asset.title ?? asset.storage_url?.split("/").pop() ?? asset.external_url ?? "Untitled";
}
