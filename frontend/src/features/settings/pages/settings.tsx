import { useAuthStore } from "@/store/auth";

import { ProfileCard } from "../components/profile-card";

export function SettingsPage() {
  const user = useAuthStore((state) => state.user);

  if (!user) {
    return <div className="playlist-empty">Sign in to manage your settings.</div>;
  }

  return (
    <div>
      <div className="playlist-detail-header">
        <div className="playlist-detail-title">Settings</div>
      </div>
      <div className="settings-content">
        <ProfileCard user={user} />
      </div>
    </div>
  );
}
