import { useEffect } from "react";
import { Outlet } from "react-router";

import { authApi } from "@/api/auth";
import { useAuthStore } from "@/store/auth";

import { Sidebar } from "./sidebar/sidebar";

export function RootLayout() {
  const setUser = useAuthStore((state) => state.setUser);

  useEffect(() => {
    void (async () => {
      const { data } = await authApi.me();
      if (data) {
        setUser({ id: data.id, username: data.username, capabilities: data.capabilities });
      }
    })();
  }, [setUser]);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto bg-canvas">
        <Outlet />
      </main>
    </div>
  );
}
