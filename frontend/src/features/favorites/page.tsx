import { useState } from "react";

import { FavoritesPlaylistTab } from "@/features/playlists";
import { FavoriteSoundbitesTab } from "@/features/soundbites";
import { useAuthStore } from "@/store/auth";

type FavoritesTab = "performances" | "soundbites";

const TABS: { id: FavoritesTab; label: string }[] = [
  { id: "performances", label: "Performances" },
  { id: "soundbites", label: "Soundbites" },
];

export function FavoritesPage() {
  const user = useAuthStore((s) => s.user);
  const [activeTab, setActiveTab] = useState<FavoritesTab>("performances");

  if (!user) {
    return <div className="page-empty">Sign in to see your favorites.</div>;
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Favorites</div>
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
      {activeTab === "performances" && <FavoritesPlaylistTab userId={user.id} />}
      {activeTab === "soundbites" && <FavoriteSoundbitesTab userId={user.id} />}
    </div>
  );
}
