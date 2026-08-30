import { useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { useAuthStore } from "@/store/auth";

import { PerformancesAdminTab } from "./admin/performances";
import { SongsAdminTab } from "./admin/songs";
import { TagsAdminTab } from "./admin/tags";
import { UsersAdminTab } from "./admin/users";

type AdminTab = "users" | "songs" | "performances" | "tags";

const ALL_TABS: { id: AdminTab; label: string; capability: string }[] = [
  { id: "users", label: "Users", capability: "capabilities:manage" },
  { id: "songs", label: "Songs", capability: "songs:manage_any" },
  { id: "performances", label: "Performances", capability: "performances:manage_any" },
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
            className={`admin-tab${activeTab === tab.id ? " admin-tab--active" : ""}`}
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
        {activeTab === "songs" && <SongsAdminTab />}
        {activeTab === "performances" && <PerformancesAdminTab />}
        {activeTab === "tags" && <TagsAdminTab />}
      </div>
    </div>
  );
}
