import { TrashIcon } from "@phosphor-icons/react";
import { useRef, useState } from "react";

import { type AdminAsset, assetLabel } from "./asset-utils";

export type StagingMediaItem<K extends string> =
  | { type: "file"; file: File; kind: K }
  | { type: "link"; externalUrl: string; title?: string; kind: K };

interface MediaEditSectionProps<K extends string> {
  label: string;
  urlPlaceholder: string;
  accept: string;
  existingItems: AdminAsset[];
  pendingRemoveIds: Set<string>;
  pendingKindChanges: Map<string, K>;
  stagingItems: StagingMediaItem<K>[];
  kinds: readonly K[];
  onAddFile: (file: File) => void;
  onAddLink: (item: { externalUrl: string; title?: string }) => void;
  onRemoveExisting: (id: string) => void;
  onChangeExistingKind: (id: string, kind: K) => void;
  onRemoveStaged: (index: number) => void;
  onChangeStagedKind: (index: number, kind: K) => void;
}

interface KindSelectProps<K extends string> {
  value: string;
  kinds: readonly K[];
  onChange: (kind: K) => void;
}

function KindSelect<K extends string>({ value, kinds, onChange }: KindSelectProps<K>) {
  return (
    <select
      className="admin-kind-select"
      value={value}
      onChange={(event) => {
        const newKind = kinds.find((k) => k === event.target.value);
        if (newKind) onChange(newKind);
      }}
    >
      {kinds.map((k) => (
        <option key={k} value={k}>
          {k}
        </option>
      ))}
    </select>
  );
}

export function MediaEditSection<K extends string>({
  label,
  urlPlaceholder,
  accept,
  existingItems,
  pendingRemoveIds,
  pendingKindChanges,
  stagingItems,
  kinds,
  onAddFile,
  onAddLink,
  onRemoveExisting,
  onChangeExistingKind,
  onRemoveStaged,
  onChangeStagedKind,
}: MediaEditSectionProps<K>) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [linkUrl, setLinkUrl] = useState("");
  const [linkTitle, setLinkTitle] = useState("");

  const visible = existingItems.filter((item) => !pendingRemoveIds.has(item.asset_id));

  return (
    <div className="form-field">
      <div className="admin-link-card-header">
        <span className="form-label">{label}</span>
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
          placeholder={urlPlaceholder}
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
        accept={accept}
        className="sr-only"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) onAddFile(file);
          event.target.value = "";
        }}
      />
      {visible.map((item) => (
        <div key={item.asset_id} className="admin-audio-item">
          <KindSelect
            value={pendingKindChanges.get(item.asset_id) ?? item.kind}
            kinds={kinds}
            onChange={(kind) => {
              onChangeExistingKind(item.asset_id, kind);
            }}
          />
          <span className="admin-asset-info">
            <span className="admin-link-url">{assetLabel(item)}</span>
            {item.title && item.external_url && (
              <span className="admin-link-label">{item.external_url}</span>
            )}
          </span>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              onRemoveExisting(item.asset_id);
            }}
          >
            <TrashIcon weight="bold" />
          </button>
        </div>
      ))}
      {stagingItems.map((item, index) => (
        <div key={index} className="admin-audio-item">
          <KindSelect
            value={item.kind}
            kinds={kinds}
            onChange={(kind) => {
              onChangeStagedKind(index, kind);
            }}
          />
          <span className="admin-asset-info">
            {item.type === "file" ? (
              <span className="admin-link-url">{item.file.name}</span>
            ) : (
              <>
                <span className="admin-link-url">{item.title ?? item.externalUrl}</span>
                {item.title && <span className="admin-link-label">{item.externalUrl}</span>}
              </>
            )}
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
