import { createBrowserRouter } from "react-router";

import { RootLayout } from "@/layouts/root-layout";
import { AdminPage } from "@/pages/admin";
import { ArtistsPage } from "@/pages/artists";
import { HomePage } from "@/pages/home";
import { SearchPage } from "@/pages/search";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "admin", element: <AdminPage /> },
      { path: "artists", element: <ArtistsPage /> },
      { path: "search", element: <SearchPage /> },
    ],
  },
]);
