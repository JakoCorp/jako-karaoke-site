import { useEffect } from "react";
import { Outlet } from "react-router";

import { auth } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

import { Sidebar } from "./sidebar/sidebar";

export function RootLayout() {
  const setUser = useAuthStore((state) => state.setUser);

  useEffect(() => {
    void (async () => {
      const { data } = await auth.me();
      if (data) {
        setUser({ id: data.id, username: data.username, capabilities: data.capabilities });
      }
    })();
  }, [setUser]);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto bg-neutral-100">
        <Outlet />
      </main>
    </div>
  );
}
