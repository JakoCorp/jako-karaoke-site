import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";

import { authApi, toAuthUser } from "@/api/auth";
import type { AuthUser } from "@/api/auth";
import { formatDate } from "@/lib/format";
import { useAuthStore } from "@/store/auth";

const USERNAME_PATTERN = /^[A-Za-z0-9_.]{1,64}$/;

interface Props {
  user: AuthUser;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function UsernameForm({ user, onDone }: { user: AuthUser; onDone: () => void }) {
  const setUser = useAuthStore((state) => state.setUser);
  const [username, setUsername] = useState(user.username);
  const [error, setError] = useState<string | null>(null);

  const changeableAt =
    user.usernameChangeableAt !== null && new Date(user.usernameChangeableAt) > new Date()
      ? user.usernameChangeableAt
      : null;
  const locked = changeableAt !== null;

  const save = useMutation({
    mutationFn: async (nextUsername: string) => {
      const { data, response } = await authApi.updateMe(nextUsername);
      if (!data) throw response.status;
      return data;
    },
    onSuccess: (data) => {
      setUser(toAuthUser(data));
      onDone();
    },
    onError: (status: unknown) => {
      setError(
        status === 409
          ? "That username is already taken."
          : "Could not change your username. Please try again.",
      );
    },
  });

  function handleSubmit() {
    if (!USERNAME_PATTERN.test(username)) {
      setError("Usernames are 1 to 64 characters: letters, numbers, underscores and periods.");
      return;
    }
    setError(null);
    save.mutate(username);
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
        <label className="form-label" htmlFor="settings-username">
          Username
        </label>
        <input
          id="settings-username"
          className="form-input"
          type="text"
          value={username}
          maxLength={64}
          disabled={locked}
          onChange={(event) => {
            setUsername(event.target.value);
          }}
        />
      </div>
      <p className="settings-hint">
        {locked
          ? `You can change your username again on ${formatDate(changeableAt)}.`
          : "You can change your username once every 30 days."}
      </p>
      {error !== null && <p className="form-error">{error}</p>}
      <div className="dialog-actions">
        <Dialog.Close className="btn btn-secondary">Cancel</Dialog.Close>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={locked || save.isPending || username === user.username}
        >
          {save.isPending ? "Saving..." : "Save"}
        </button>
      </div>
    </form>
  );
}

export function UsernameDialog({ user, open, onOpenChange }: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <UsernameForm
            user={user}
            onDone={() => {
              onOpenChange(false);
            }}
          />
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
