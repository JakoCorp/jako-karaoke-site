import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { UserPerformancePlayEntry, UserSummary } from "@/api/users";
import { ConfirmDialog } from "@/components/confirm-dialog";
import {
  useRemoveUserPerformancePlays,
  useRemoveUserSoundbitePlays,
  useUserPerformancePlays,
  useUserSoundbitePlays,
} from "@/hooks/api/users";
import { errorMessage } from "@/lib/error-message";
import { formatDate } from "@/lib/format";

import { PlayList, PlayPager, type PlayListItem } from "../components/play-list";

interface Props {
  user: UserSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

type PlayKind = "performances" | "soundbites";

const PLAY_KINDS: { id: PlayKind; label: string }[] = [
  { id: "performances", label: "Performances" },
  { id: "soundbites", label: "Soundbites" },
];

/** Which plays a confirmation will remove: those on one item, or `null` for all of them. */
interface Removal {
  itemId: string | null;
}

function performanceLabel(entry: UserPerformancePlayEntry): string {
  return entry.title ?? entry.songs.map((song) => song.title).join(" / ");
}

function playMeta(entry: { user_play_count: number; last_played_at: string }): string {
  return `${entry.user_play_count.toLocaleString()} plays · last ${formatDate(entry.last_played_at)}`;
}

function PlaysList({ user }: { user: UserSummary }) {
  const [kind, setKind] = useState<PlayKind>("performances");
  const [page, setPage] = useState(1);
  const [removal, setRemoval] = useState<Removal | null>(null);
  const performancePlays = useUserPerformancePlays(user.id, page, kind === "performances");
  const soundbitePlays = useUserSoundbitePlays(user.id, page, kind === "soundbites");
  const removePerformancePlays = useRemoveUserPerformancePlays();
  const removeSoundbitePlays = useRemoveUserSoundbitePlays();

  const plays = kind === "performances" ? performancePlays : soundbitePlays;
  const remove = kind === "performances" ? removePerformancePlays : removeSoundbitePlays;
  const noun = kind === "performances" ? "performance" : "soundbite";
  const lastPage = plays.data ? Math.max(1, Math.ceil(plays.data.total / plays.data.per_page)) : 1;

  const items: PlayListItem[] =
    kind === "performances"
      ? (performancePlays.data?.items ?? []).map((entry) => ({
          id: entry.id,
          title: performanceLabel(entry),
          meta: playMeta(entry),
          onRemove: () => {
            setRemoval({ itemId: entry.id });
          },
        }))
      : (soundbitePlays.data?.items ?? []).map((entry) => ({
          id: entry.id,
          title: entry.title,
          meta: playMeta(entry),
          onRemove: () => {
            setRemoval({ itemId: entry.id });
          },
        }));

  function selectKind(next: PlayKind) {
    setKind(next);
    setPage(1);
    setRemoval(null);
  }

  function confirmRemoval() {
    if (removal === null) return;
    const options = {
      onSuccess: () => {
        setRemoval(null);
        setPage(1);
      },
    };
    if (kind === "performances") {
      removePerformancePlays.mutate({ userId: user.id, performanceId: removal.itemId }, options);
    } else {
      removeSoundbitePlays.mutate({ userId: user.id, soundbiteId: removal.itemId }, options);
    }
  }

  return (
    <div className="dialog-form">
      <Dialog.Title className="text-base font-semibold">Plays by @{user.username}</Dialog.Title>
      <div className="dialog-actions">
        {PLAY_KINDS.map((option) => (
          <button
            key={option.id}
            type="button"
            className={kind === option.id ? "btn btn-primary" : "btn btn-secondary"}
            aria-pressed={kind === option.id}
            onClick={() => {
              selectKind(option.id);
            }}
          >
            {option.label}
          </button>
        ))}
      </div>
      {plays.isLoading && <p className="admin-empty">Loading…</p>}
      {plays.isError && (
        <p className="form-error">{errorMessage(plays.error, "Could not load plays.")}</p>
      )}
      {plays.data?.items.length === 0 && <p className="admin-empty">No counted plays.</p>}
      <PlayList items={items} />
      <PlayPager page={page} lastPage={lastPage} onPageChange={setPage}>
        <button
          type="button"
          className="btn btn-danger"
          disabled={!plays.data || plays.data.total === 0}
          onClick={() => {
            setRemoval({ itemId: null });
          }}
        >
          Remove all
        </button>
      </PlayPager>
      <ConfirmDialog
        open={removal !== null}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) setRemoval(null);
        }}
        onConfirm={confirmRemoval}
        title={removal?.itemId === null ? "Remove all plays?" : "Remove plays?"}
        description={
          removal?.itemId === null
            ? `Every counted ${noun} play by @${user.username} will be removed.`
            : `Plays by @${user.username} on this ${noun} will be removed.`
        }
        confirmLabel="Remove"
        variant="danger"
        isPending={remove.isPending}
        error={remove.isError ? errorMessage(remove.error, "Could not remove plays.") : null}
      />
    </div>
  );
}

/** Dialog for an admin to review and reverse the counted plays of a user. */
export function PlaysDialog({ user, open, onOpenChange }: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup dialog-popup--wide">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <PlaysList user={user} />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
