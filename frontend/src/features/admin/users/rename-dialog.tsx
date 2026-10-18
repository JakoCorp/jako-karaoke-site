import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import type { UserSummary } from "@/api/users";
import { useUpdateUsername } from "@/hooks/api/users";
import { errorMessage } from "@/lib/error-message";
import { USERNAME_PATTERN, USERNAME_RULES_MESSAGE } from "@/lib/username";

interface Props {
  user: UserSummary;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRenamed: (user: UserSummary) => void;
}

function RenameForm({ user, onDone }: { user: UserSummary; onDone: (user: UserSummary) => void }) {
  const update = useUpdateUsername();
  const [username, setUsername] = useState(user.username);
  const [validationError, setValidationError] = useState<string | null>(null);

  const error =
    validationError ??
    (update.isError ? errorMessage(update.error, "Could not change the username.") : null);

  function handleSubmit() {
    if (!USERNAME_PATTERN.test(username)) {
      setValidationError(USERNAME_RULES_MESSAGE);
      return;
    }
    setValidationError(null);
    update.mutate({ userId: user.id, username }, { onSuccess: onDone });
  }

  return (
    <form
      className="dialog-form"
      onSubmit={(event) => {
        event.preventDefault();
        handleSubmit();
      }}
    >
      <Dialog.Title className="text-base font-semibold">Change username</Dialog.Title>
      <div className="form-field">
        <label className="form-label" htmlFor="admin-rename-username">
          Username
        </label>
        <input
          id="admin-rename-username"
          className="form-input"
          type="text"
          value={username}
          maxLength={64}
          onChange={(event) => {
            setUsername(event.target.value);
          }}
        />
      </div>
      <p className="form-hint">
        The user cannot change their username again for 30 days after this change.
      </p>
      {error !== null && <p className="form-error">{error}</p>}
      <div className="dialog-actions">
        <Dialog.Close className="btn btn-secondary">Cancel</Dialog.Close>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={update.isPending || username === user.username}
        >
          {update.isPending ? "Saving..." : "Save"}
        </button>
      </div>
    </form>
  );
}

/** Dialog for an admin to change another user's username. */
export function RenameDialog({ user, open, onOpenChange, onRenamed }: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <RenameForm
            user={user}
            onDone={(renamed) => {
              onRenamed(renamed);
              onOpenChange(false);
            }}
          />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
