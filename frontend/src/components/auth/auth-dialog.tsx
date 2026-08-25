import { Dialog } from "@base-ui/react";
import { UserIcon, XIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { LoginForm } from "./login-form";
import { OAuthButtons } from "./oauth-buttons";
import { RegisterForm } from "./register-form";

export function AuthDialog() {
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<"login" | "register">("login");

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
            <Dialog.Title className="text-base font-semibold">
              {mode === "login" ? "Sign In" : "Register"}
            </Dialog.Title>
            <div className="auth-tabs">
              <button
                className={`auth-tab${mode === "login" ? " auth-tab--active" : ""}`}
                onClick={() => {
                  setMode("login");
                }}
              >
                Sign In
              </button>
              <button
                className={`auth-tab${mode === "register" ? " auth-tab--active" : ""}`}
                onClick={() => {
                  setMode("register");
                }}
              >
                Register
              </button>
            </div>
            {mode === "login" ? (
              <LoginForm
                onSuccess={() => {
                  setOpen(false);
                }}
              />
            ) : (
              <RegisterForm
                onSuccess={() => {
                  setOpen(false);
                }}
              />
            )}
            <OAuthButtons />
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
