import { useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { useAuthStore } from "@/store/auth";

import { UsersAdminTab } from "./admin/users";

type AdminTab = "users";

const TABS: { id: AdminTab; label: string }[] = [{ id: "users", label: "Users" }];

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

  return (
    <div className="admin-page">
      <div className="admin-tabs">
        {TABS.map((tab) => (
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
      <div className="admin-content">{activeTab === "users" && <UsersAdminTab />}</div>
    </div>
  );
}
