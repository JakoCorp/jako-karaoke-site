import { useState } from "react";

import { FavoritesPlaylistTab } from "@/features/playlists";
import { useAuthStore } from "@/store/auth";

type FavoritesTab = "playlist";

const TABS: { id: FavoritesTab; label: string }[] = [{ id: "playlist", label: "Performances" }];

export function FavoritesPage() {
  const user = useAuthStore((s) => s.user);
  const [activeTab, setActiveTab] = useState<FavoritesTab>("playlist");

  if (!user) {
    return <div className="playlist-empty">Sign in to see your favorites.</div>;
  }

  return (
    <div>
      <div className="playlist-detail-header">
        <div className="playlist-detail-title">Favorites</div>
      </div>
      <div className="favorites-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={
              activeTab === tab.id ? "favorites-tab favorites-tab--active" : "favorites-tab"
            }
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {activeTab === "playlist" && <FavoritesPlaylistTab userId={user.id} />}
    </div>
  );
}
