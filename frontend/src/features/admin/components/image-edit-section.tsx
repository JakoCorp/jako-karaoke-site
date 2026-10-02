import { TrashIcon } from "@phosphor-icons/react";
import { useRef, useState } from "react";

interface ExistingImage {
  asset_id: string;
  storage_url?: string | null;
  external_url?: string | null;
  kind: string;
}

export type StagingImageItem =
  | { type: "file"; file: File }
  | { type: "link"; externalUrl: string; title?: string };

interface ImageEditSectionProps<K extends string> {
  existingImages: ExistingImage[];
  pendingRemoveIds: Set<string>;
  pendingKindChanges: Map<string, K>;
  stagingItems: StagingImageItem[];
  kinds: readonly K[];
  onAddFile: (file: File) => void;
  onAddLink: (item: { externalUrl: string; title?: string }) => void;
  onRemoveExisting: (id: string) => void;
  onChangeExistingKind: (id: string, kind: K) => void;
  onRemoveStaged: (index: number) => void;
}

export function ImageEditSection<K extends string>({
  existingImages,
  pendingRemoveIds,
  pendingKindChanges,
  stagingItems,
  kinds,
  onAddFile,
  onAddLink,
  onRemoveExisting,
  onChangeExistingKind,
  onRemoveStaged,
}: ImageEditSectionProps<K>) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [linkUrl, setLinkUrl] = useState("");
  const [linkTitle, setLinkTitle] = useState("");

  const visible = existingImages.filter((img) => !pendingRemoveIds.has(img.asset_id));

  return (
    <div className="form-field">
      <div className="admin-link-card-header">
        <span className="form-label">Images</span>
        <div className="admin-link-card-actions">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              fileInputRef.current?.click();
            }}
          >
            Upload file
          </button>
        </div>
      </div>
      <div className="admin-link-input-row">
        <input
          type="url"
          className="form-input"
          placeholder="External image URL"
          value={linkUrl}
          onChange={(event) => {
            setLinkUrl(event.target.value);
          }}
        />
        <input
          type="text"
          className="form-input"
          placeholder="Title (optional)"
          value={linkTitle}
          onChange={(event) => {
            setLinkTitle(event.target.value);
          }}
        />
        <button
          type="button"
          className="btn btn-secondary"
          disabled={linkUrl.trim() === ""}
          onClick={() => {
            const url = linkUrl.trim();
            if (!url) return;
            onAddLink({ externalUrl: url, title: linkTitle.trim() || undefined });
            setLinkUrl("");
            setLinkTitle("");
          }}
        >
          Add link
        </button>
      </div>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        className="sr-only"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) onAddFile(file);
          event.target.value = "";
        }}
      />
      {visible.length > 0 && (
        <div className="admin-image-list">
          {visible.map((img) => (
            <div key={img.asset_id} className="admin-image-item">
              <img src={img.storage_url ?? img.external_url ?? undefined} alt={img.kind} />
              <select
                className="admin-kind-select"
                value={pendingKindChanges.get(img.asset_id) ?? img.kind}
                onChange={(event) => {
                  const newKind = kinds.find((k) => k === event.target.value);
                  if (!newKind) return;
                  onChangeExistingKind(img.asset_id, newKind);
                }}
              >
                {kinds.map((k) => (
                  <option key={k} value={k}>
                    {k}
                  </option>
                ))}
              </select>
              <button
                type="button"
                className="admin-image-delete"
                onClick={() => {
                  onRemoveExisting(img.asset_id);
                }}
              >
                <TrashIcon weight="bold" />
              </button>
            </div>
          ))}
        </div>
      )}
      {stagingItems.map((item, index) => (
        <div key={index} className="admin-audio-item">
          <span className="admin-link-url text-sm text-fg-muted">
            {item.type === "file" ? item.file.name : (item.title ?? item.externalUrl)}
          </span>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              onRemoveStaged(index);
            }}
          >
            <TrashIcon weight="bold" />
          </button>
        </div>
      ))}
    </div>
  );
}
