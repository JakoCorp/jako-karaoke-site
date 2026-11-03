import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { SoundbiteResponse } from "@/api/soundbites";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { useSoundbitePlayers } from "@/hooks/api/soundbites";
import { useRemoveUserSoundbitePlays } from "@/hooks/api/users";
import { errorMessage } from "@/lib/error-message";
import { formatDate } from "@/lib/format";

import { PlayList, PlayPager } from "../components/play-list";

interface Props {
  soundbite: SoundbiteResponse;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface Removal {
  userId: string;
  username: string;
}

function PlayersList({ soundbite }: { soundbite: SoundbiteResponse }) {
  const [page, setPage] = useState(1);
  const [removal, setRemoval] = useState<Removal | null>(null);
  const players = useSoundbitePlayers(soundbite.id, page);
  const remove = useRemoveUserSoundbitePlays();

  const lastPage = players.data
    ? Math.max(1, Math.ceil(players.data.total / players.data.per_page))
    : 1;
  const rankOffset = players.data ? (players.data.page - 1) * players.data.per_page : 0;

  function confirmRemoval() {
    if (removal === null) return;
    remove.mutate(
      { userId: removal.userId, soundbiteId: soundbite.id },
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
      <Dialog.Title className="text-base font-semibold">Plays by user</Dialog.Title>
      <p className="admin-empty">{soundbite.play_count.toLocaleString()} plays in total</p>
      {players.isLoading && <p className="admin-empty">Loading…</p>}
      {players.isError && (
        <p className="form-error">{errorMessage(players.error, "Could not load plays.")}</p>
      )}
      {players.data?.items.length === 0 && <p className="admin-empty">No counted plays.</p>}
      <PlayList
        items={(players.data?.items ?? []).map((player, index) => ({
          id: player.id,
          title: `${rankOffset + index + 1}. @${player.username}`,
          meta: `${player.play_count.toLocaleString()} plays · last ${formatDate(player.last_played_at)}`,
          onRemove: () => {
            setRemoval({ userId: player.id, username: player.username });
          },
        }))}
      />
      <PlayPager page={page} lastPage={lastPage} onPageChange={setPage} />
      <ConfirmDialog
        open={removal !== null}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) setRemoval(null);
        }}
        onConfirm={confirmRemoval}
        title="Remove plays?"
        description={`Plays by @${removal?.username ?? ""} on this soundbite (${soundbite.title}) will be removed.`}
        confirmLabel="Remove"
        variant="danger"
        isPending={remove.isPending}
        error={remove.isError ? errorMessage(remove.error, "Could not remove plays.") : null}
      />
    </div>
  );
}

/** Dialog for an admin to see who played a soundbite and reverse suspicious plays. */
export function SoundbitePlayersDialog({ soundbite, open, onOpenChange }: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup dialog-popup--wide">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <PlayersList soundbite={soundbite} />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
