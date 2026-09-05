import { Popover } from "@base-ui/react";
import { UserIcon } from "@phosphor-icons/react";
import { Link } from "react-router";

import { authApi } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

export function UserMenu() {
  const user = useAuthStore((state) => state.user);
  const setUser = useAuthStore((state) => state.setUser);
  const hasCapability = useAuthStore((state) => state.hasCapability);

  async function handleLogout() {
    await authApi.logout();
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
            {hasCapability("capabilities:manage") && (
              <Link to="/admin" className="btn w-full btn-secondary">
                Admin
              </Link>
            )}
            <button
              onClick={() => {
                void handleLogout();
              }}
              className="btn w-full btn-secondary"
            >
              Sign Out
            </button>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
