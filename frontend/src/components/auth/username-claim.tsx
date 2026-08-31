import { Dialog } from "@base-ui/react";
import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router";

import { authApi } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

export function UsernameClaim() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const setUser = useAuthStore((state) => state.setUser);

  const suggested = searchParams.get("username");

  const [ready, setReady] = useState(false);
  const [username, setUsername] = useState(() => suggested ?? "");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (suggested === null) return;

    const verify = async () => {
      const { response } = await authApi.checkPending();
      if (response.ok) {
        setReady(true);
      } else {
        void navigate("/", { replace: true });
      }
    };

    void verify();
  }, [suggested, navigate]);

  if (suggested === null || !ready) return null;

  async function handleSubmit() {
    setSubmitting(true);
    setError(null);

    const { data, response } = await authApi.claim({ username });

    if (data) {
      setUser({ id: data.id, username: data.username, capabilities: data.capabilities });
      void navigate("/", { replace: true });
    } else if (response?.status === 409) {
      setError("Username already taken, please choose another.");
    } else {
      setError("Something went wrong. Please try again.");
    }

    setSubmitting(false);
  }

  return (
    <Dialog.Root open onOpenChange={() => {}}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup">
          <div className="flex flex-col gap-4">
            <Dialog.Title className="text-base font-semibold">Choose a username</Dialog.Title>
            <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
              This is how you&apos;ll appear on the site. You can keep your provider handle or pick
              something new.
            </p>
            <form
              onSubmit={(event) => {
                event.preventDefault();
                void handleSubmit();
              }}
              className="flex flex-col gap-4"
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
