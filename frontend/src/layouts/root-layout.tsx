import { useEffect } from "react";
import { useState } from "react";
import { Outlet } from "react-router";

import { authApi } from "@/api/auth";
import { MusicPlayer } from "@/features/player";
import { useAuthStore } from "@/store/auth";

import { HeaderMobile } from "./header-mobile";
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

  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />

      <div className="flex min-w-0 flex-1 flex-col">
        <HeaderMobile onOpenSidebar={() => setSidebarOpen(true)} />

        <main className="flex-1 overflow-y-auto bg-canvas">
          <Outlet />
        </main>

        <MusicPlayer />
      </div>
    </div>
  );
}
