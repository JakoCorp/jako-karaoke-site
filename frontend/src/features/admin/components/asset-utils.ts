export interface AdminAsset {
  asset_id: string;
  title?: string | null;
  storage_url?: string | null;
  external_url?: string | null;
  kind: string;
}

export function assetLabel(asset: AdminAsset): string {
  return asset.title ?? asset.storage_url?.split("/").pop() ?? asset.external_url ?? "Untitled";
}
