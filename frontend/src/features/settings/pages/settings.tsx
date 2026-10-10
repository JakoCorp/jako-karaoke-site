import { useAuthStore } from "@/store/auth";

import { DownloadSettingsCard } from "../components/download-settings-card";
import { ProfileCard } from "../components/profile-card";

export function SettingsPage() {
  const user = useAuthStore((state) => state.user);

  if (!user) {
    return <div className="page-empty">Sign in to manage your settings.</div>;
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Settings</div>
      </div>
      <div className="settings-content">
        <ProfileCard user={user} />
        <DownloadSettingsCard />
      </div>
    </div>
  );
}
