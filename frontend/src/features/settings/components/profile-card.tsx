import { useState } from "react";

import type { AuthUser } from "@/api/auth";

import { AvatarPicker } from "./avatar-picker";
import { UsernameDialog } from "./username-dialog";

interface Props {
  user: AuthUser;
}

export function ProfileCard({ user }: Props) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [avatarError, setAvatarError] = useState<string | null>(null);

  return (
    <div className="settings-card">
      <AvatarPicker user={user} onError={setAvatarError} />
      <div className="settings-card-body">
        <button
          type="button"
          className="settings-username-btn"
          data-tooltip="Change username"
          onClick={() => {
            setDialogOpen(true);
          }}
        >
          {user.username}
        </button>
        {avatarError !== null && <p className="form-error">{avatarError}</p>}
      </div>
      <UsernameDialog user={user} open={dialogOpen} onOpenChange={setDialogOpen} />
    </div>
  );
}
