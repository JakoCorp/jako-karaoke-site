import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { UserPlayEntry, UserSummary } from "@/api/users";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useRemoveUserPlays, useUserPlays } from "@/hooks/api/users";
import { errorMessage } from "@/lib/error-message";
import { formatDate } from "@/lib/format";

import { PlayList, PlayPager } from "../components/play-list";

interface Props {
  user: UserSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Which plays a confirmation will remove: one performance, or `null` for all of them. */
interface Removal {
  performanceId: string | null;
}

function entryLabel(entry: UserPlayEntry): string {
  return entry.title ?? entry.songs.map((song) => song.title).join(" / ");
}

function PlaysList({ user }: { user: UserSummary }) {
  const [page, setPage] = useState(1);
  const [removal, setRemoval] = useState<Removal | null>(null);
  const plays = useUserPlays(user.id, page);
  const remove = useRemoveUserPlays();

  const lastPage = plays.data ? Math.max(1, Math.ceil(plays.data.total / plays.data.per_page)) : 1;

  function confirmRemoval() {
    if (removal === null) return;
    remove.mutate(
      { userId: user.id, performanceId: removal.performanceId },
      {
        onSuccess: () => {
          setRemoval(null);
          setPage(1);
        },
      },
    );
  }

  return (
    <div className="dialog-form">
      <Dialog.Title className="text-base font-semibold">Plays by @{user.username}</Dialog.Title>
      {plays.isLoading && <p className="admin-empty">Loading…</p>}
      {plays.isError && (
        <p className="form-error">{errorMessage(plays.error, "Could not load plays.")}</p>
      )}
      {plays.data?.items.length === 0 && <p className="admin-empty">No counted plays.</p>}
      <PlayList
        items={(plays.data?.items ?? []).map((entry) => ({
          id: entry.id,
          title: entryLabel(entry),
          meta: `${entry.user_play_count.toLocaleString()} plays · last ${formatDate(entry.last_played_at)}`,
          onRemove: () => {
            setRemoval({ performanceId: entry.id });
          },
        }))}
      />
      <PlayPager page={page} lastPage={lastPage} onPageChange={setPage}>
        <button
          type="button"
          className="btn btn-danger"
          disabled={!plays.data || plays.data.total === 0}
          onClick={() => {
            setRemoval({ performanceId: null });
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
        title={removal?.performanceId === null ? "Remove all plays?" : "Remove plays?"}
        description={
          removal?.performanceId === null
            ? `Every counted play by @${user.username} will be removed.`
            : `Plays by @${user.username} on this performance will be removed.`
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
