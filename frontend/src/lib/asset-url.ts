interface AssetUrls {
  storage_url?: string | null;
  external_url?: string | null;
}

/** Returns the internally hosted URL of an asset, falling back to its external URL. */
export function resolveAssetUrl(asset: AssetUrls | null | undefined): string | undefined {
  return asset?.storage_url ?? asset?.external_url ?? undefined;
}
