import { Dialog } from "@base-ui/react";
import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

import { artistsApi } from "@/api/artists";
import { performancesApi, type PerformanceTagKind } from "@/api/performances";
import { songsApi } from "@/api/songs";
import { tagsApi } from "@/api/tags";
import { performanceKeys } from "@/hooks/api/performances";
import { parseImportCsv, type ParsedPerformanceImportRow } from "@/lib/parse-import-csv";

interface ResolvedRow {
  row: ParsedPerformanceImportRow;
  song_ids: string[];
  singer_ids: string[];
  tags: Array<{ tag_id: string; kind: PerformanceTagKind }>;
}

type ResolveResult = { ok: true; resolved: ResolvedRow[] } | { ok: false; errors: string[] };

async function resolveEntities(rows: ParsedPerformanceImportRow[]): Promise<ResolveResult> {
  const uniqueSingerNames = [...new Set(rows.flatMap((r) => r.singer_names))];
  const uniqueSongTitles = [...new Set(rows.flatMap((r) => r.song_titles))];
  const uniqueTagNames = [...new Set(rows.flatMap((r) => r.tags.map((t) => t.name)))];

  const [tagsResult, artistResults, songResults] = await Promise.all([
    tagsApi.list(),
    Promise.all(uniqueSingerNames.map((name) => artistsApi.list({ q: name, per_page: 200 }))),
    Promise.all(uniqueSongTitles.map((title) => songsApi.list({ q: title, per_page: 200 }))),
  ]);

  if (tagsResult.error) return { ok: false, errors: ["Failed to fetch tags."] };

  const tagMap = new Map<string, string>();
  for (const tag of tagsResult.data) {
    tagMap.set(tag.name.toLowerCase(), tag.id);
  }

  const artistMap = new Map<string, string>();
  const missingArtists: string[] = [];
  for (let i = 0; i < uniqueSingerNames.length; i++) {
    const name = uniqueSingerNames[i]!;
    const result = artistResults[i];
    if (result?.error) return { ok: false, errors: [`Failed to look up artist "${name}".`] };
    const match = result?.data?.items?.find((a) => a.name.toLowerCase() === name.toLowerCase());
    if (match) {
      artistMap.set(name.toLowerCase(), match.id);
    } else {
      missingArtists.push(name);
    }
  }

  const songMap = new Map<string, string>();
  const missingSongs: string[] = [];
  for (let i = 0; i < uniqueSongTitles.length; i++) {
    const title = uniqueSongTitles[i]!;
    const result = songResults[i];
    if (result?.error) return { ok: false, errors: [`Failed to look up song "${title}".`] };
    const match = result?.data?.items?.find((s) => s.title.toLowerCase() === title.toLowerCase());
    if (match) {
      songMap.set(title.toLowerCase(), match.id);
    } else {
      missingSongs.push(title);
    }
  }

  const missingTags: string[] = [];
  for (const tagName of uniqueTagNames) {
    if (!tagMap.has(tagName.toLowerCase())) {
      missingTags.push(tagName);
    }
  }

  const errors: string[] = [];
  if (missingArtists.length > 0) {
    errors.push(`Missing artists: ${missingArtists.map((n) => `"${n}"`).join(", ")}.`);
  }
  if (missingSongs.length > 0) {
    errors.push(`Missing songs: ${missingSongs.map((t) => `"${t}"`).join(", ")}.`);
  }
  if (missingTags.length > 0) {
    errors.push(`Missing tags: ${missingTags.map((n) => `"${n}"`).join(", ")}.`);
  }
  if (errors.length > 0) return { ok: false, errors };

  const resolved: ResolvedRow[] = rows.map((row) => ({
    row,
    song_ids: row.song_titles.map((t) => songMap.get(t.toLowerCase())!),
    singer_ids: row.singer_names.map((n) => artistMap.get(n.toLowerCase())!),
    tags: row.tags.map((t) => ({
      tag_id: tagMap.get(t.name.toLowerCase())!,
      kind: t.kind,
    })),
  }));

  return { ok: true, resolved };
}

type ImportPhase =
  | { phase: "idle" }
  | { phase: "parse_error"; errors: string[] }
  | { phase: "resolving" }
  | { phase: "resolve_error"; errors: string[] }
  | { phase: "ready"; resolved: ResolvedRow[] }
  | { phase: "importing"; done: number; total: number }
  | { phase: "import_error"; message: string; done: number; total: number }
  | { phase: "success"; count: number };

interface ImportPerformancesDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function ImportPerformancesDialog({
  open,
  onClose,
  onSuccess,
}: ImportPerformancesDialogProps) {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [state, setState] = useState<ImportPhase>({ phase: "idle" });
  const [selectedFileName, setSelectedFileName] = useState<string | null>(null);

  function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    setSelectedFileName(file.name);

    const reader = new FileReader();
    reader.addEventListener("load", (e) => {
      const text = e.target?.result;
      if (typeof text !== "string") return;

      const parseResult = parseImportCsv(text);
      if (!parseResult.ok) {
        setState({ phase: "parse_error", errors: parseResult.errors });
        return;
      }

      setState({ phase: "resolving" });
      void (async () => {
        const result = await resolveEntities(parseResult.rows);
        if (!result.ok) {
          setState({ phase: "resolve_error", errors: result.errors });
        } else {
          setState({ phase: "ready", resolved: result.resolved });
        }
      })();
    });
    reader.readAsText(file);
  }

  async function handleImport() {
    if (state.phase !== "ready") return;
    const { resolved } = state;
    setState({ phase: "importing", done: 0, total: resolved.length });

    for (let i = 0; i < resolved.length; i++) {
      const { row, song_ids, singer_ids, tags } = resolved[i]!;
      const { error } = await performancesApi.create({
        performance_date: row.performance_date,
        stream_number: row.stream_number,
        performance_number: row.performance_number,
        title: row.title,
        duration: row.duration,
        stream_time: row.stream_time,
        song_ids,
        singer_ids,
        tags,
      });
      if (error) {
        const message =
          typeof error === "object" && error !== null && "error" in error
            ? String((error as { error: unknown }).error)
            : `Failed on performance ${String(i + 1)}.`;
        setState({ phase: "import_error", message, done: i, total: resolved.length });
        return;
      }
      setState({ phase: "importing", done: i + 1, total: resolved.length });
    }

    void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
    setState({ phase: "success", count: resolved.length });
    onSuccess();
  }

  function resetToIdle() {
    setState({ phase: "idle" });
    setSelectedFileName(null);
    if (fileInputRef.current) fileInputRef.current.value = "";
  }

  const isImporting = state.phase === "importing";

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !isImporting) onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Title className="admin-dialog-title">Import Performances</Dialog.Title>

          <div className="admin-dialog-form">
            {(state.phase === "idle" ||
              state.phase === "parse_error" ||
              state.phase === "resolve_error") && (
              <div className="form-field">
                <label className="form-label" htmlFor="import-csv-file">
                  CSV file
                </label>
                <div className="admin-link-row">
                  <button
                    type="button"
                    className="btn shrink-0 btn-secondary"
                    onClick={() => fileInputRef.current?.click()}
                  >
                    Choose file
                  </button>
                  <span
                    className="form-input flex-1"
                    style={{ color: selectedFileName ? undefined : "var(--color-fg-muted)" }}
                  >
                    {selectedFileName ?? "No file chosen"}
                  </span>
                </div>
                <input
                  id="import-csv-file"
                  ref={fileInputRef}
                  type="file"
                  accept=".csv"
                  className="absolute size-0 overflow-hidden opacity-0"
                  onChange={handleFileChange}
                />
              </div>
            )}

            {state.phase === "parse_error" && (
              <div className="form-error">
                <ul>
                  {state.errors.map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                </ul>
              </div>
            )}

            {state.phase === "resolving" && <p className="admin-empty">Resolving entities…</p>}

            {state.phase === "resolve_error" && (
              <div className="form-error">
                <ul>
                  {state.errors.map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                </ul>
              </div>
            )}

            {state.phase === "ready" && (
              <p>
                {String(state.resolved.length)} performance
                {state.resolved.length !== 1 ? "s" : ""} ready to import.
              </p>
            )}

            {state.phase === "importing" && (
              <p className="admin-empty">
                Importing {String(state.done)}/{String(state.total)}…
              </p>
            )}

            {state.phase === "import_error" && (
              <>
                <p className="form-error">{state.message}</p>
                {state.done > 0 && (
                  <p className="admin-empty">
                    {String(state.done)} of {String(state.total)} created before the error.
                  </p>
                )}
              </>
            )}

            {state.phase === "success" && (
              <p>
                Successfully imported {String(state.count)} performance
                {state.count !== 1 ? "s" : ""}.
              </p>
            )}
          </div>

          <div className="admin-dialog-actions">
            {state.phase === "ready" && (
              <>
                <button type="button" className="btn btn-secondary" onClick={resetToIdle}>
                  Back
                </button>
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={() => {
                    void handleImport();
                  }}
                >
                  Import
                </button>
              </>
            )}

            {state.phase === "importing" && (
              <button type="button" className="btn btn-secondary" disabled>
                Importing…
              </button>
            )}

            {(state.phase === "idle" ||
              state.phase === "parse_error" ||
              state.phase === "resolve_error") && (
              <button type="button" className="btn btn-secondary" onClick={onClose}>
                Cancel
              </button>
            )}

            {(state.phase === "import_error" || state.phase === "success") && (
              <button type="button" className="btn btn-secondary" onClick={onClose}>
                Close
              </button>
            )}
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
