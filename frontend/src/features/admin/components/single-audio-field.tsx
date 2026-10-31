import { TrashIcon } from "@phosphor-icons/react";
import { useRef } from "react";

interface SingleAudioFieldProps {
  label: string;
  /** Name of the audio stored now, shown until a replacement file is chosen. */
  currentName?: string;
  /** The chosen file that has not been uploaded yet. */
  file: File | null;
  onChange: (file: File | null) => void;
}

/** File picker for a resource that has a single audio file, such as a soundbite. */
export function SingleAudioField({ label, currentName, file, onChange }: SingleAudioFieldProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const hasAudio = file !== null || currentName !== undefined;

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
            {hasAudio ? "Replace file" : "Choose file"}
          </button>
        </div>
      </div>
      <input
        ref={fileInputRef}
        type="file"
        accept="audio/*"
        className="sr-only"
        onChange={(event) => {
          const chosen = event.target.files?.[0];
          if (chosen) onChange(chosen);
          event.target.value = "";
        }}
      />
      {file !== null ? (
        <div className="admin-audio-item">
          <span className="admin-asset-info">
            <span className="admin-link-url">{file.name}</span>
          </span>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => {
              onChange(null);
            }}
          >
            <TrashIcon weight="bold" />
          </button>
        </div>
      ) : (
        currentName !== undefined && (
          <div className="admin-audio-item">
            <span className="admin-asset-info">
              <span className="admin-link-url">{currentName}</span>
            </span>
          </div>
        )
      )}
    </div>
  );
}
