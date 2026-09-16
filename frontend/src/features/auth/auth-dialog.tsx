import { Dialog } from "@base-ui/react";
import { UserIcon, XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { authApi } from "@/api/auth";
import discordIcon from "@/assets/Discord-Clyde-Blurple.svg";
import twitchIcon from "@/assets/Twitch-Glitch.svg";
import { useAuthStore } from "@/store/auth";

export function AuthDialog() {
  const [open, setOpen] = useState(false);
  const setUser = useAuthStore((state) => state.setUser);

  async function handleDevLogin() {
    const { data } = await authApi.devLogin();
    if (data) {
      setUser({ id: data.id, username: data.username, capabilities: data.capabilities });
      setOpen(false);
    }
  }

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger className="auth-trigger">
        <UserIcon size={16} weight="fill" className="shrink-0" />
        Sign In
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <Dialog.Close className="dialog-close" aria-label="Close">
            <XIcon size={16} />
          </Dialog.Close>
          <div className="dialog-form">
            <Dialog.Title className="text-base font-semibold">Sign In</Dialog.Title>
            <div className="admin-link-list">
              <a
                href={`${import.meta.env.VITE_API_URL ?? ""}/auth/twitch`}
                className="btn oauth-btn btn-secondary"
              >
                <img src={twitchIcon} alt="" className="h-4.5 w-auto shrink-0" />
                Continue with Twitch
              </a>
              <a
                href={`${import.meta.env.VITE_API_URL ?? ""}/auth/discord`}
                className="btn oauth-btn btn-secondary"
              >
                <img src={discordIcon} alt="" className="h-4.5 w-auto shrink-0" />
                Continue with Discord
              </a>
              {import.meta.env.DEV && (
                <button
                  className="btn btn-secondary"
                  onClick={() => {
                    void handleDevLogin();
                  }}
                >
                  Dev Login
                </button>
              )}
            </div>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
