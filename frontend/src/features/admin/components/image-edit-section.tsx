import { TrashIcon } from "@phosphor-icons/react";
import { useRef } from "react";

interface ExistingImage {
  id: string;
  public_url: string;
  kind: string;
}

interface ImageEditSectionProps<K extends string> {
  existingImages: ExistingImage[];
  pendingRemoveIds: Set<string>;
  pendingKindChanges: Map<string, K>;
  stagingFiles: File[];
  kinds: readonly K[];
  onFileSelect: (file: File) => void;
  onRemoveExisting: (id: string) => void;
  onChangeExistingKind: (id: string, kind: K) => void;
  onRemoveStaged: (index: number) => void;
}

export function ImageEditSection<K extends string>({
  existingImages,
  pendingRemoveIds,
  pendingKindChanges,
  stagingFiles,
  kinds,
  onFileSelect,
  onRemoveExisting,
  onChangeExistingKind,
  onRemoveStaged,
}: ImageEditSectionProps<K>) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const visible = existingImages.filter((img) => !pendingRemoveIds.has(img.id));

  return (
    <div className="form-field">
      <div className="admin-link-card-header">
        <span className="form-label">Images</span>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={() => {
            fileInputRef.current?.click();
          }}
        >
          Add image
        </button>
      </div>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        className="sr-only"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) onFileSelect(file);
          event.target.value = "";
        }}
      />
      {visible.length > 0 && (
        <div className="admin-image-list">
          {visible.map((img) => (
            <div key={img.id} className="admin-image-item">
              <img src={img.public_url} alt={img.kind} />
              <select
                className="admin-kind-select"
                value={pendingKindChanges.get(img.id) ?? img.kind}
                onChange={(event) => {
                  const newKind = kinds.find((k) => k === event.target.value);
                  if (!newKind) return;
                  onChangeExistingKind(img.id, newKind);
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
                  onRemoveExisting(img.id);
                }}
              >
                <TrashIcon weight="bold" />
              </button>
            </div>
          ))}
        </div>
      )}
      {stagingFiles.map((file, index) => (
        <div key={index} className="admin-audio-item">
          <span className="text-sm text-fg-muted">{file.name}</span>
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
