import { Dialog } from "@base-ui/react";
import { UserIcon, XIcon } from "@phosphor-icons/react";
import { useState } from "react";

export function AuthDialog() {
  const [open, setOpen] = useState(false);

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
          <div className="flex flex-col gap-4">
            <Dialog.Title className="text-base font-semibold">Sign In</Dialog.Title>
            <div className="flex flex-col gap-2">
              <a
                href="/auth/twitch"
                className="oauth-btn text-white"
                style={{ backgroundColor: "#9146FF" }}
              >
                Continue with Twitch
              </a>
              <a
                href="/auth/discord"
                className="oauth-btn text-white"
                style={{ backgroundColor: "#5865F2" }}
              >
                Continue with Discord
              </a>
            </div>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
