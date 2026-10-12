import { resolveAssetUrl } from "@/lib/asset-url";

import { type AdminAsset, assetLabel } from "./asset-utils";

interface ImageListProps {
  images: AdminAsset[];
}

/** Read only list of images with their thumbnail, label and kind. */
export function ImageList({ images }: ImageListProps) {
  if (images.length === 0) return null;

  return (
    <div className="admin-detail-section">
      <span className="admin-detail-label">Images</span>
      <div className="admin-image-list">
        {images.map((image) => (
          <div key={image.asset_id} className="admin-image-item">
            <img className="admin-image-thumb" src={resolveAssetUrl(image)} alt="" />
            <span className="admin-asset-info">
              <span className="admin-link-url text-sm">{assetLabel(image)}</span>
            </span>
            <span className="admin-pill-kind">{image.kind}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
