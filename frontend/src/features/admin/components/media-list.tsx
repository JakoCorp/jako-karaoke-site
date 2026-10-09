import { resolveAssetUrl } from "@/lib/asset-url";

import { type AdminAsset, assetLabel } from "./asset-utils";

interface MediaListProps {
  label: string;
  items: AdminAsset[];
}

/** Read only list of audio or video assets with their kind and a link to the file. */
export function MediaList({ label, items }: MediaListProps) {
  if (items.length === 0) return null;

  return (
    <div className="admin-detail-section">
      <span className="admin-detail-label">{label}</span>
      {items.map((item) => (
        <div key={item.asset_id} className="admin-audio-item">
          <span className="admin-pill-kind">{item.kind}</span>
          <a
            href={resolveAssetUrl(item)}
            target="_blank"
            rel="noreferrer"
            className="admin-link-url"
          >
            {assetLabel(item)}
          </a>
        </div>
      ))}
    </div>
  );
}
