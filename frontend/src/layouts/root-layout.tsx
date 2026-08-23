import { Outlet } from "react-router";

import { Sidebar } from "./sidebar/sidebar";

export function RootLayout() {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto bg-neutral-100">
        <Outlet />
      </main>
    </div>
  );
}
