import { useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { useAuthStore } from "@/store/auth";

import { ArtistsAdminTab } from "./artists/tab";
import { PerformancesAdminTab } from "./performances/tab";
import { SongsAdminTab } from "./songs/tab";
import { SoundbitesAdminTab } from "./soundbites/tab";
import { TagsAdminTab } from "./tags/tab";
import { UsersAdminTab } from "./users/tab";

type AdminTab = "users" | "performances" | "soundbites" | "songs" | "artists" | "tags";

const ALL_TABS: { id: AdminTab; label: string; capability: string }[] = [
  { id: "users", label: "Users", capability: "capabilities:manage" },
  { id: "performances", label: "Performances", capability: "performances:manage_any" },
  { id: "soundbites", label: "Soundbites", capability: "soundbites:manage_any" },
  { id: "songs", label: "Songs", capability: "songs:manage_any" },
  { id: "artists", label: "Artists", capability: "artists:manage_any" },
  { id: "tags", label: "Tags", capability: "tags:manage_any" },
];

export function AdminPage() {
  const navigate = useNavigate();
  const user = useAuthStore((state) => state.user);
  const hasCapability = useAuthStore((state) => state.hasCapability);
  const [activeTab, setActiveTab] = useState<AdminTab>("users");

  useEffect(() => {
    if (user !== null && !hasCapability("capabilities:manage")) {
      void navigate("/", { replace: true });
    }
  }, [user, hasCapability, navigate]);

  if (user === null || !hasCapability("capabilities:manage")) return null;

  const visibleTabs = ALL_TABS.filter((tab) => hasCapability(tab.capability));

  return (
    <div className="admin-page">
      <div className="admin-tabs">
        {visibleTabs.map((tab) => (
          <button
            key={tab.id}
            className={activeTab === tab.id ? "admin-tab admin-tab--active" : "admin-tab"}
            onClick={() => {
              setActiveTab(tab.id);
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <div className="admin-content">
        {activeTab === "users" && <UsersAdminTab />}
        {activeTab === "performances" && <PerformancesAdminTab />}
        {activeTab === "soundbites" && <SoundbitesAdminTab />}
        {activeTab === "songs" && <SongsAdminTab />}
        {activeTab === "artists" && <ArtistsAdminTab />}
        {activeTab === "tags" && <TagsAdminTab />}
      </div>
    </div>
  );
}
