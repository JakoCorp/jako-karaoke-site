import { createBrowserRouter } from "react-router";

import { AdminPage } from "@/features/admin";
import { ArtistsPage } from "@/features/artists";
import { FavoritesPage } from "@/features/favorites";
import { HomePage } from "@/features/home";
import { MyPlaylistsPage, PlaylistDetailPage, PublicPlaylistsPage } from "@/features/playlists";
import { SearchPage } from "@/features/search";
import { RootLayout } from "@/layouts/root-layout";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "admin", element: <AdminPage /> },
      { path: "artists", element: <ArtistsPage /> },
      { path: "favorites", element: <FavoritesPage /> },
      { path: "my-playlists", element: <MyPlaylistsPage /> },
      { path: "playlist/:id", element: <PlaylistDetailPage /> },
      { path: "public-playlists", element: <PublicPlaylistsPage /> },
      { path: "search", element: <SearchPage /> },
    ],
  },
]);
