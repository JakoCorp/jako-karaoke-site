import { Popover } from "@base-ui/react";
import { UserIcon } from "@phosphor-icons/react";

import { auth } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

export function AccountPopover() {
  const user = useAuthStore((state) => state.user);
  const setUser = useAuthStore((state) => state.setUser);

  async function handleLogout() {
    await auth.logout();
    setUser(null);
  }

  if (!user) return null;

  return (
    <Popover.Root>
      <Popover.Trigger className="auth-trigger">
        <UserIcon size={16} weight="fill" className="shrink-0" />
        {user.username}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner side="top" align="center" sideOffset={8}>
          <Popover.Popup className="account-popup">
            <p className="account-popup-user">@{user.username}</p>
            <button
              onClick={() => {
                void handleLogout();
              }}
              className="btn btn-secondary w-full"
            >
              Sign Out
            </button>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
