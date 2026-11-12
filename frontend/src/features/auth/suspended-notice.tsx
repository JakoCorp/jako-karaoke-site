import { Dialog } from "@base-ui/react";
import { useNavigate, useSearchParams } from "react-router";

/** Tells a suspended user why sign in failed. Shown when the callback redirects with `?suspended=true`. */
export function SuspendedNotice() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  if (searchParams.get("suspended") !== "true") return null;

  return (
    <Dialog.Root
      open
      onOpenChange={() => {
        void navigate("/", { replace: true });
      }}
    >
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <div className="dialog-form">
            <Dialog.Title className="text-base font-semibold">Account suspended</Dialog.Title>
            <p className="text-sm text-fg-muted">
              This account has been suspended and cannot sign in.
            </p>
            <div className="dialog-actions">
              <Dialog.Close className="btn btn-primary">OK</Dialog.Close>
            </div>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
