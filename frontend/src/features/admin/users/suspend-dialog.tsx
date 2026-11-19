import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { AdminUserSummary } from "@/api/users";
import { useSuspendUser } from "@/hooks/api/users";
import { errorMessage } from "@/lib/error-message";

const MAX_REASON_LENGTH = 500;

interface Props {
  user: AdminUserSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuspended: (user: AdminUserSummary) => void;
}

function SuspendForm({
  user,
  onDone,
}: {
  user: AdminUserSummary;
  onDone: (user: AdminUserSummary) => void;
}) {
  const suspend = useSuspendUser();
  const [reason, setReason] = useState("");

  const error = suspend.isError ? errorMessage(suspend.error, "Could not suspend the user.") : null;

  return (
    <form
      className="dialog-form"
      onSubmit={(event) => {
        event.preventDefault();
        suspend.mutate({ userId: user.id, reason: reason.trim() }, { onSuccess: onDone });
      }}
    >
      <Dialog.Title className="text-base font-semibold">Suspend @{user.username}?</Dialog.Title>
      <p className="form-hint">
        They will be signed out and cannot sign in until the suspension is lifted. Their plays,
        playlists and favorites are kept.
      </p>
      <div className="form-field">
        <label className="form-label" htmlFor="admin-suspend-reason">
          Reason
        </label>
        <textarea
          id="admin-suspend-reason"
          className="form-input admin-textarea"
          rows={3}
          value={reason}
          maxLength={MAX_REASON_LENGTH}
          onChange={(event) => {
            setReason(event.target.value);
          }}
        />
      </div>
      {error !== null && <p className="form-error">{error}</p>}
      <div className="dialog-actions">
        <Dialog.Close className="btn btn-secondary">Cancel</Dialog.Close>
        <button
          type="submit"
          className="btn btn-danger"
          disabled={suspend.isPending || reason.trim() === ""}
        >
          {suspend.isPending ? "Saving…" : "Suspend"}
        </button>
      </div>
    </form>
  );
}

/** Dialog for an admin to suspend a user and record why. */
export function SuspendDialog({ user, open, onOpenChange, onSuspended }: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <SuspendForm
            user={user}
            onDone={(suspended) => {
              onSuspended(suspended);
              onOpenChange(false);
            }}
          />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
