import { useRef, useState } from "react";

import type { DownloadSettings } from "@/api/settings";
import { FILENAME_TOKENS, renderFilename } from "@/features/download";
import type { FilenameValues } from "@/features/download";
import { useUpdateUserSettings, useUserSettings } from "@/hooks/api/settings";

const MAX_TEMPLATE_LENGTH = 128;

const SAMPLE_VALUES: FilenameValues = {
  title: "Example Song",
  singer: "Singer",
  original_artist: "Original Artist",
  song: "Example Song",
  date: "2025-01-31",
  stream_number: "1",
  performance_number: "3",
};

const TAG_OPTIONS = [
  { key: "include_cover_art", label: "Include cover art" },
  { key: "include_lyrics", label: "Include lyrics" },
  { key: "include_date", label: "Include performance date" },
  { key: "include_singers", label: "Include singer" },
  { key: "include_original_artists", label: "Include original artist" },
] as const satisfies readonly { key: keyof DownloadSettings; label: string }[];

function DownloadSettingsForm({ initial }: { initial: DownloadSettings }) {
  const save = useUpdateUserSettings();
  const templateInput = useRef<HTMLInputElement>(null);
  const [draft, setDraft] = useState(initial);

  const saved = save.data?.download ?? initial;
  const isDirty = JSON.stringify(draft) !== JSON.stringify(saved);
  const isValid = draft.filename_template.trim() !== "";

  function insertToken(token: string) {
    const input = templateInput.current;
    const text = `{${token}}`;
    const template = draft.filename_template;
    const start = input?.selectionStart ?? template.length;
    const end = input?.selectionEnd ?? start;
    const next = template.slice(0, start) + text + template.slice(end);
    if (next.length > MAX_TEMPLATE_LENGTH) return;

    setDraft({ ...draft, filename_template: next });
    const caret = start + text.length;
    requestAnimationFrame(() => {
      input?.focus();
      input?.setSelectionRange(caret, caret);
    });
  }

  return (
    <section className="settings-section">
      <div className="settings-section-title">Downloads</div>

      <div className="form-field">
        <label className="form-label" htmlFor="download-filename-template">
          Filename template
        </label>
        <input
          id="download-filename-template"
          ref={templateInput}
          className="form-input"
          value={draft.filename_template}
          maxLength={MAX_TEMPLATE_LENGTH}
          onChange={(event) => {
            setDraft({ ...draft, filename_template: event.target.value });
          }}
        />
        <div className="settings-token-row">
          {FILENAME_TOKENS.map(({ token, description }) => (
            <button
              key={token}
              type="button"
              className="settings-token-btn"
              data-tooltip={description}
              onClick={() => {
                insertToken(token);
              }}
            >
              {`{${token}}`}
            </button>
          ))}
        </div>
        <p className="settings-preview">
          Preview: {renderFilename(draft.filename_template, SAMPLE_VALUES, "mp3")}
        </p>
      </div>

      <div className="form-field">
        <div className="form-label">Metadata Tags:</div>
        <div className="settings-checkbox-list">
          {TAG_OPTIONS.map(({ key, label }) => (
            <label key={key} className="form-checkbox">
              <input
                type="checkbox"
                checked={draft[key]}
                onChange={(event) => {
                  setDraft({ ...draft, [key]: event.target.checked });
                }}
              />
              {label}
            </label>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-1">
        <p className="settings-hint">
          Singer and original artist are combined into the artist tag as &quot;singer - original
          artist&quot;.
        </p>
        <p className="settings-hint">
          Tags are written to MP3, M4A and AAC files, other formats download unchanged.
        </p>
      </div>

      {save.isError && <p className="form-error">Could not save your settings. Try again.</p>}

      <div>
        <button
          type="button"
          className="btn btn-primary"
          disabled={!isDirty || !isValid || save.isPending}
          onClick={() => {
            save.mutate({ download: draft });
          }}
        >
          {save.isPending ? "Saving..." : "Save"}
        </button>
      </div>
    </section>
  );
}

/** Settings card for download filename and tag preferences. */
export function DownloadSettingsCard() {
  const { data, isError } = useUserSettings();

  if (isError) {
    return <p className="form-error">Could not load your download settings.</p>;
  }
  if (!data) return null;

  return <DownloadSettingsForm initial={data.download} />;
}
