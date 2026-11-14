import { Dialog } from "@base-ui/react";
import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";

import { authApi, toAuthUser } from "@/api/auth";
import { unwrap } from "@/api/client";
import { errorMessage } from "@/lib/error-message";
import { useAuthStore } from "@/store/auth";

/** Lets a new user pick a username. Shown when the callback redirects with `?auth=claim`. */
export function UsernameClaim() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const setUser = useAuthStore((state) => state.setUser);

  const isClaiming = searchParams.get("auth") === "claim";

  const [ready, setReady] = useState(false);
  const [username, setUsername] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!isClaiming) return;

    const loadPending = async () => {
      const { data } = await authApi.getPending();
      if (data) {
        setUsername(data.suggested_username);
        setReady(true);
      } else {
        void navigate("/", { replace: true });
      }
    };

    void loadPending();
  }, [isClaiming, navigate]);

  if (!isClaiming || !ready) return null;

  async function handleSubmit() {
    setSubmitting(true);
    setError(null);

    try {
      const data = unwrap(await authApi.claim({ username }));
      setUser(toAuthUser(data));
      void navigate("/", { replace: true });
    } catch (claimError) {
      setError(errorMessage(claimError, "Something went wrong. Please try again."));
    }

    setSubmitting(false);
  }

  return (
    <Dialog.Root open onOpenChange={() => {}}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <div className="dialog-form">
            <Dialog.Title className="text-base font-semibold">Choose a username</Dialog.Title>
            <p className="text-sm text-fg-muted">
              This is how you&apos;ll appear on the site. You can keep your provider handle or pick
              something new.
            </p>
            <form
              onSubmit={(event) => {
                event.preventDefault();
                void handleSubmit();
              }}
              className="dialog-form"
            >
              <div className="form-field">
                <label className="form-label" htmlFor="claim-username">
                  Username
                </label>
                <input
                  id="claim-username"
                  className="form-input"
                  type="text"
                  value={username}
                  onChange={(event) => {
                    setUsername(event.target.value);
                  }}
                  required
                  minLength={1}
                  maxLength={64}
                />
              </div>
              {error !== null && <p className="form-error">{error}</p>}
              <button type="submit" className="btn btn-primary" disabled={submitting}>
                {submitting ? "Saving..." : "Continue"}
              </button>
            </form>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
