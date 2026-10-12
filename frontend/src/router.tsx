import { createBrowserRouter } from "react-router";

import { AdminPage } from "@/features/admin";
import { ArtistsPage } from "@/features/artists";
import { FavoritesPage } from "@/features/favorites";
import { HomePage } from "@/features/home";
import { LocalPage } from "@/features/local";
import { PerformanceDetailPage } from "@/features/performances";
import { MyPlaylistsPage, PlaylistDetailPage, PublicPlaylistsPage } from "@/features/playlists";
import { SearchPage } from "@/features/search";
import { SettingsPage } from "@/features/settings";
import { SongDetailPage } from "@/features/songs";
import { RootErrorBoundary } from "@/layouts/root-error-boundary";
import { RootLayout } from "@/layouts/root-layout";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    errorElement: <RootErrorBoundary />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "admin", element: <AdminPage /> },
      { path: "artists", element: <ArtistsPage /> },
      { path: "favorites", element: <FavoritesPage /> },
      { path: "local", element: <LocalPage /> },
      { path: "my-playlists", element: <MyPlaylistsPage /> },
      { path: "performance/:id", element: <PerformanceDetailPage /> },
      { path: "playlist/:id", element: <PlaylistDetailPage /> },
      { path: "public-playlists", element: <PublicPlaylistsPage /> },
      { path: "search", element: <SearchPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "song/:id", element: <SongDetailPage /> },
    ],
  },
]);
