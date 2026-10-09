import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import {
  AUDIO_KINDS,
  PERFORMANCE_TAG_KINDS,
  VIDEO_KINDS,
  performancesApi,
  type AudioKind,
  type PerformanceSummary,
  type PerformanceTagKind,
  type VideoKind,
} from "@/api/performances";
import { tagsApi } from "@/api/tags";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useArtists } from "@/hooks/api/artists";
import { performanceKeys, usePerformance } from "@/hooks/api/performances";
import { useSongs } from "@/hooks/api/songs";
import { tagKeys, useTags } from "@/hooks/api/tags";
import { formatDate, formatStreamTime, parseStreamTime } from "@/lib/format";
import { applyAll } from "@/lib/staging";

import { MediaEditSection, type StagingMediaItem } from "../components/media-edit-section";
import { MediaList } from "../components/media-list";
import { ItemPicker, TagPicker, type TagAssignment } from "../components/pickers";
import { resolveTagAssignments } from "../components/tag-utils";

export function PerformanceDetailPanel({
  performance,
  onClose,
}: {
  performance: PerformanceSummary | null;
  onClose: () => void;
}) {
  const isCreating = performance === null;
  const [isEditing, setIsEditing] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editDate, setEditDate] = useState("");
  const [editStreamNumber, setEditStreamNumber] = useState(1);
  const [editPerformanceNumber, setEditPerformanceNumber] = useState(1);
  const [editStreamTimeInput, setEditStreamTimeInput] = useState("");
  const [editSongIds, setEditSongIds] = useState<string[]>([]);
  const [editSingerIds, setEditSingerIds] = useState<string[]>([]);
  const [editTags, setEditTags] = useState<TagAssignment<PerformanceTagKind>[]>([]);
  const [editLyrics, setEditLyrics] = useState("");
  const [stagingAddAudio, setStagingAddAudio] = useState<StagingMediaItem<AudioKind>[]>([]);
  const [pendingRemoveAudioIds, setPendingRemoveAudioIds] = useState<Set<string>>(new Set());
  const [pendingAudioKindChanges, setPendingAudioKindChanges] = useState<Map<string, AudioKind>>(
    new Map(),
  );
  const [stagingAddVideo, setStagingAddVideo] = useState<StagingMediaItem<VideoKind>[]>([]);
  const [pendingRemoveVideoIds, setPendingRemoveVideoIds] = useState<Set<string>>(new Set());
  const [pendingVideoKindChanges, setPendingVideoKindChanges] = useState<Map<string, VideoKind>>(
    new Map(),
  );
  const [formError, setFormError] = useState<string | null>(null);

  const queryClient = useQueryClient();
  const isFormOpen = isCreating || isEditing;

  const { data: performanceDetail } = usePerformance(performance?.id ?? "", !isCreating);
  const { data: allSongs } = useSongs({ per_page: 200 }, isFormOpen);
  const { data: allArtists } = useArtists({ per_page: 200 }, isFormOpen);
  const { data: allTags } = useTags(isFormOpen);

  const createMutation = useMutation({
    mutationFn: async () => {
      const resolvedTags = await resolveTagAssignments(editTags, async (name) => {
        const { data, error: tagError } = await tagsApi.create({ name });
        if (tagError) throw tagError;
        if (!data) throw new Error("Tag creation returned no data.");
        return data.id;
      });
      const { data, error: apiError } = await performancesApi.create({
        performance_date: editDate,
        stream_number: editStreamNumber,
        performance_number: editPerformanceNumber,
        stream_time: parseStreamTime(editStreamTimeInput) ?? undefined,
        song_ids: editSongIds,
        singer_ids: editSingerIds,
        tags: resolvedTags,
        title: editTitle.trim() !== "" ? editTitle.trim() : null,
        lyrics: editLyrics.trim() !== "" ? editLyrics.trim() : null,
      });
      if (apiError) throw apiError;
      if (!data) throw new Error("Performance creation returned no data.");
      await applyAll(stagingAddAudio, (item) =>
        item.type === "file"
          ? performancesApi.uploadAudio(data.id, item.file, item.kind)
          : performancesApi.addAudioLink(data.id, {
              external_url: item.externalUrl,
              kind: item.kind,
              title: item.title ?? null,
            }),
      );
      await applyAll(stagingAddVideo, (item) =>
        item.type === "file"
          ? performancesApi.uploadVideo(data.id, item.file, item.kind)
          : performancesApi.addVideoLink(data.id, {
              external_url: item.externalUrl,
              kind: item.kind,
              title: item.title ?? null,
            }),
      );
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      onClose();
    },
    onError: () => {
      setFormError("Failed to create performance.");
    },
  });

  const updateMutation = useMutation({
    mutationFn: async () => {
      if (!performance) return;
      const resolvedTags = await resolveTagAssignments(editTags, async (name) => {
        const { data, error: tagError } = await tagsApi.create({ name });
        if (tagError) throw tagError;
        if (!data) throw new Error("Tag creation returned no data.");
        return data.id;
      });
      const { error: apiError } = await performancesApi.update(performance.id, {
        performance_date: editDate,
        stream_number: editStreamNumber,
        performance_number: editPerformanceNumber,
        stream_time: parseStreamTime(editStreamTimeInput) ?? undefined,
        song_ids: editSongIds,
        singer_ids: editSingerIds,
        tags: resolvedTags,
        title: editTitle.trim() !== "" ? editTitle.trim() : null,
      });
      if (apiError) throw apiError;
      await applyAll(pendingAudioKindChanges, ([audioId, kind]) =>
        performancesApi.updateAudioKind(performance.id, audioId, kind),
      );
      await applyAll(pendingVideoKindChanges, ([videoId, kind]) =>
        performancesApi.updateVideoKind(performance.id, videoId, kind),
      );
      await applyAll(stagingAddAudio, (item) =>
        item.type === "file"
          ? performancesApi.uploadAudio(performance.id, item.file, item.kind)
          : performancesApi.addAudioLink(performance.id, {
              external_url: item.externalUrl,
              kind: item.kind,
              title: item.title ?? null,
            }),
      );
      await applyAll(stagingAddVideo, (item) =>
        item.type === "file"
          ? performancesApi.uploadVideo(performance.id, item.file, item.kind)
          : performancesApi.addVideoLink(performance.id, {
              external_url: item.externalUrl,
              kind: item.kind,
              title: item.title ?? null,
            }),
      );
      await applyAll(pendingRemoveAudioIds, (id) =>
        performancesApi.deleteAudio(performance.id, id),
      );
      await applyAll(pendingRemoveVideoIds, (id) =>
        performancesApi.deleteVideo(performance.id, id),
      );
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setStagingAddAudio([]);
      setStagingAddVideo([]);
      setPendingRemoveAudioIds(new Set());
      setPendingRemoveVideoIds(new Set());
      setPendingAudioKindChanges(new Map());
      setPendingVideoKindChanges(new Map());
      setIsEditing(false);
      setFormError(null);
    },
    onError: () => {
      setFormError("Failed to update performance.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      if (!performance) return;
      const { error: apiError } = await performancesApi.delete(performance.id);
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: performanceKeys.all() });
      onClose();
    },
  });

  function startEditing() {
    if (!performanceDetail) return;
    setEditTitle(performanceDetail.title ?? "");
    setEditDate(performanceDetail.performance_date);
    setEditStreamNumber(performanceDetail.stream_number);
    setEditPerformanceNumber(performanceDetail.performance_number);
    setEditStreamTimeInput(
      performanceDetail.stream_time != null ? formatStreamTime(performanceDetail.stream_time) : "",
    );
    setEditSongIds(performanceDetail.songs.map((song) => song.id));
    setEditSingerIds(performanceDetail.singers.map((singer) => singer.id));
    setEditTags(
      performanceDetail.tags.map((tag) => ({
        type: "existing" as const,
        tagId: tag.id,
        kind: PERFORMANCE_TAG_KINDS.find((k) => k === tag.kind) ?? "misc",
      })),
    );
    setStagingAddAudio([]);
    setStagingAddVideo([]);
    setPendingRemoveAudioIds(new Set());
    setPendingRemoveVideoIds(new Set());
    setPendingAudioKindChanges(new Map());
    setPendingVideoKindChanges(new Map());
    setFormError(null);
    setIsEditing(true);
  }

  function cancelEditing() {
    setStagingAddAudio([]);
    setStagingAddVideo([]);
    setPendingRemoveAudioIds(new Set());
    setPendingRemoveVideoIds(new Set());
    setPendingAudioKindChanges(new Map());
    setPendingVideoKindChanges(new Map());
    setIsEditing(false);
    setFormError(null);
  }

  function toggleSong(songId: string) {
    setEditSongIds((previous) =>
      previous.includes(songId) ? previous.filter((id) => id !== songId) : [...previous, songId],
    );
  }

  function toggleSinger(singerId: string) {
    setEditSingerIds((previous) =>
      previous.includes(singerId)
        ? previous.filter((id) => id !== singerId)
        : [...previous, singerId],
    );
  }

  function addExistingTag(tagId: string) {
    setEditTags((previous) => [...previous, { type: "existing", tagId, kind: "misc" }]);
  }

  function addPendingTag(name: string) {
    setEditTags((previous) => [...previous, { type: "pending", pendingName: name, kind: "misc" }]);
  }

  function removeTag(key: string) {
    setEditTags((previous) =>
      previous.filter((a) => (a.type === "existing" ? a.tagId !== key : a.pendingName !== key)),
    );
  }

  function changeTagKind(key: string, kind: PerformanceTagKind) {
    setEditTags((previous) =>
      previous.map((a) => {
        if (a.type === "existing" && a.tagId === key) return { ...a, kind };
        if (a.type === "pending" && a.pendingName === key) return { ...a, kind };
        return a;
      }),
    );
  }

  const existingAudio = isEditing ? (performanceDetail?.audio ?? []) : [];
  const existingVideo = isEditing ? (performanceDetail?.video ?? []) : [];

  const hasPrimaryAudio =
    existingAudio
      .filter((audio) => !pendingRemoveAudioIds.has(audio.asset_id))
      .some((audio) => (pendingAudioKindChanges.get(audio.asset_id) ?? audio.kind) === "primary") ||
    stagingAddAudio.some((item) => item.kind === "primary");
  const defaultAudioKind: AudioKind = hasPrimaryAudio ? "misc" : "primary";

  function changeExistingAudioKind(id: string, kind: AudioKind) {
    setPendingAudioKindChanges((previous) => {
      const next = new Map(previous);
      if (kind === "primary") {
        for (const audio of existingAudio) {
          if (audio.asset_id !== id) next.set(audio.asset_id, "misc");
        }
      }
      return next.set(id, kind);
    });
    if (kind === "primary") {
      setStagingAddAudio((previous) =>
        previous.map((item) => (item.kind === "primary" ? { ...item, kind: "misc" } : item)),
      );
    }
  }

  function changeStagedAudioKind(index: number, kind: AudioKind) {
    setStagingAddAudio((previous) =>
      previous.map((item, i) => {
        if (i === index) return { ...item, kind };
        if (kind === "primary" && item.kind === "primary") return { ...item, kind: "misc" };
        return item;
      }),
    );
  }

  if (isFormOpen) {
    const isPending = isCreating ? createMutation.isPending : updateMutation.isPending;
    const displayTitle = isCreating
      ? "New performance"
      : (performance.title ?? formatDate(performance.performance_date));
    return (
      <>
        <div className="admin-panel-header">
          <h3 className="admin-panel-title">{displayTitle}</h3>
          <div className="admin-tag-confirm">
            <button
              className="btn btn-secondary"
              type="button"
              onClick={isCreating ? onClose : cancelEditing}
            >
              Cancel
            </button>
            <button
              className="btn btn-primary"
              type="button"
              disabled={
                isPending || editDate === "" || editStreamNumber < 1 || editPerformanceNumber < 1
              }
              onClick={() => {
                if (isCreating) {
                  createMutation.mutate();
                } else {
                  updateMutation.mutate();
                }
              }}
            >
              {isPending ? (isCreating ? "Creating…" : "Saving…") : isCreating ? "Create" : "Save"}
            </button>
          </div>
        </div>
        <div className="admin-panel-scroll">
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-date">
              Performance date
            </label>
            <input
              id="perf-edit-date"
              className="form-input"
              type="date"
              value={editDate}
              onChange={(event) => {
                setEditDate(event.target.value);
              }}
              required
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-stream">
              Stream number
            </label>
            <input
              id="perf-edit-stream"
              className="form-input"
              type="number"
              min={1}
              value={editStreamNumber}
              onChange={(event) => {
                setEditStreamNumber(Number(event.target.value));
              }}
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-number">
              Performance number
            </label>
            <input
              id="perf-edit-number"
              className="form-input"
              type="number"
              min={1}
              value={editPerformanceNumber}
              onChange={(event) => {
                setEditPerformanceNumber(Number(event.target.value));
              }}
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-stream-time">
              Stream time (optional)
            </label>
            <input
              id="perf-edit-stream-time"
              className="form-input"
              type="text"
              placeholder="1:23:45"
              value={editStreamTimeInput}
              onChange={(event) => {
                setEditStreamTimeInput(event.target.value);
              }}
            />
          </div>
          <div className="form-field">
            <label className="form-label" htmlFor="perf-edit-title">
              Title (optional)
            </label>
            <input
              id="perf-edit-title"
              className="form-input"
              type="text"
              value={editTitle}
              onChange={(event) => {
                setEditTitle(event.target.value);
              }}
            />
          </div>
          <ItemPicker
            label="Songs"
            allItems={allSongs?.items ?? []}
            selectedIds={editSongIds}
            onToggle={toggleSong}
            searchPlaceholder="Search songs…"
            getLabel={(song) => song.title}
            renderDropdownItem={(song) => (
              <>
                {song.title}
                {song.artists.length > 0 && (
                  <span className="admin-item-sub">
                    {" "}
                    — {song.artists.map((artist) => artist.name).join(", ")}
                  </span>
                )}
              </>
            )}
          />
          <ItemPicker
            label="Singers"
            allItems={allArtists?.items ?? []}
            selectedIds={editSingerIds}
            onToggle={toggleSinger}
            searchPlaceholder="Search singers…"
            getLabel={(artist) => artist.name}
          />
          <TagPicker
            allTags={allTags ?? []}
            selected={editTags}
            kinds={PERFORMANCE_TAG_KINDS}
            onAddExisting={addExistingTag}
            onAddPending={addPendingTag}
            onRemove={removeTag}
            onKindChange={changeTagKind}
          />
          {isCreating && (
            <div className="form-field">
              <label className="form-label" htmlFor="perf-lyrics">
                Lyrics (optional)
              </label>
              <textarea
                id="perf-lyrics"
                className="form-input admin-textarea"
                value={editLyrics}
                onChange={(event) => {
                  setEditLyrics(event.target.value);
                }}
                rows={4}
              />
            </div>
          )}
          <MediaEditSection
            label="Audio"
            urlPlaceholder="External audio URL"
            accept="audio/*,video/mp4"
            existingItems={existingAudio}
            pendingRemoveIds={pendingRemoveAudioIds}
            pendingKindChanges={pendingAudioKindChanges}
            stagingItems={stagingAddAudio}
            kinds={AUDIO_KINDS}
            onAddFile={(file) => {
              setStagingAddAudio((prev) => [
                ...prev,
                { type: "file", file, kind: defaultAudioKind },
              ]);
            }}
            onAddLink={({ externalUrl, title }) => {
              setStagingAddAudio((prev) => [
                ...prev,
                { type: "link", externalUrl, title, kind: defaultAudioKind },
              ]);
            }}
            onRemoveExisting={(id) => {
              setPendingRemoveAudioIds((prev) => new Set([...prev, id]));
            }}
            onChangeExistingKind={changeExistingAudioKind}
            onRemoveStaged={(index) => {
              setStagingAddAudio((prev) => prev.filter((_, i) => i !== index));
            }}
            onChangeStagedKind={changeStagedAudioKind}
          />
          <MediaEditSection
            label="Video"
            urlPlaceholder="External video URL"
            accept="video/*"
            existingItems={existingVideo}
            pendingRemoveIds={pendingRemoveVideoIds}
            pendingKindChanges={pendingVideoKindChanges}
            stagingItems={stagingAddVideo}
            kinds={VIDEO_KINDS}
            onAddFile={(file) => {
              setStagingAddVideo((prev) => [...prev, { type: "file", file, kind: "misc" }]);
            }}
            onAddLink={({ externalUrl, title }) => {
              setStagingAddVideo((prev) => [
                ...prev,
                { type: "link", externalUrl, title, kind: "misc" },
              ]);
            }}
            onRemoveExisting={(id) => {
              setPendingRemoveVideoIds((prev) => new Set([...prev, id]));
            }}
            onChangeExistingKind={(id, kind) => {
              setPendingVideoKindChanges((prev) => new Map(prev).set(id, kind));
            }}
            onRemoveStaged={(index) => {
              setStagingAddVideo((prev) => prev.filter((_, i) => i !== index));
            }}
            onChangeStagedKind={(index, kind) => {
              setStagingAddVideo((prev) =>
                prev.map((item, i) => (i === index ? { ...item, kind } : item)),
              );
            }}
          />
          {formError !== null && <p className="form-error">{formError}</p>}
        </div>
      </>
    );
  }

  const displayTitle = performance.title ?? formatDate(performance.performance_date);

  return (
    <>
      <div className="admin-panel-header">
        <h3 className="admin-panel-title">{displayTitle}</h3>
        <div className="admin-tag-confirm">
          <button
            className="btn btn-secondary"
            onClick={startEditing}
            disabled={!performanceDetail}
          >
            Edit
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => {
              setDeleteOpen(true);
            }}
          >
            Delete
          </button>
        </div>
      </div>
      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={(nextOpen) => {
          setDeleteOpen(nextOpen);
          if (!nextOpen) deleteMutation.reset();
        }}
        onConfirm={() => {
          deleteMutation.mutate();
        }}
        variant="danger"
        isPending={deleteMutation.isPending}
        error={deleteMutation.isError ? "Failed to delete performance." : null}
      />

      {performanceDetail && (
        <div className="admin-panel-scroll">
          <div className="admin-detail-section">
            <span className="admin-detail-label">Date</span>
            <span className="admin-empty">{formatDate(performanceDetail.performance_date)}</span>
          </div>
          <div className="admin-detail-section">
            <span className="admin-detail-label">Stream</span>
            <span className="admin-empty">
              S{performanceDetail.stream_number} #{performanceDetail.performance_number}
              {performanceDetail.stream_time != null &&
                ` · ${formatStreamTime(performanceDetail.stream_time)}`}
            </span>
          </div>
          {performanceDetail.songs.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Songs</span>
              <div className="admin-pills">
                {performanceDetail.songs.map((song) => (
                  <span key={song.id} className="admin-pill-display">
                    {song.title}
                  </span>
                ))}
              </div>
            </div>
          )}
          {performanceDetail.singers.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Singers</span>
              <div className="admin-pills">
                {performanceDetail.singers.map((singer) => (
                  <span key={singer.id} className="admin-pill-display">
                    {singer.name}
                  </span>
                ))}
              </div>
            </div>
          )}
          {performanceDetail.tags.length > 0 && (
            <div className="admin-detail-section">
              <span className="admin-detail-label">Tags</span>
              <div className="admin-pills">
                {performanceDetail.tags.map((tag) => (
                  <span key={tag.id} className="admin-pill-display">
                    {tag.name}
                    <span className="admin-pill-kind">{tag.kind}</span>
                  </span>
                ))}
              </div>
            </div>
          )}
          <MediaList label="Audio" items={performanceDetail.audio} />
          <MediaList label="Video" items={performanceDetail.video} />
        </div>
      )}
    </>
  );
}
