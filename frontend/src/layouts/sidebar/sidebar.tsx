import logoUrl from "@/assets/Baji.factions.Industry.svg";
import { AccountPopover } from "@/components/auth/account-popover";
import { AuthDialog } from "@/components/auth/auth-dialog";
import { useAuthStore } from "@/store/auth";

import { NavItem } from "./nav-item";

export function Sidebar() {
  const user = useAuthStore((state) => state.user);

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <img src={logoUrl} alt="" aria-hidden className="sidebar-logo" />
        <span className="sidebar-brand">Karaoke Player</span>
      </div>
      <nav className="sidebar-nav">
        <NavItem to="/" label="Home" />
        <NavItem to="/search" label="Search" />
        <NavItem to="/randomsong" label="Random Songs" />
        <NavItem to="/explore" label="Explore" />
        <NavItem to="/artists" label="Artists" />
        <NavItem to="/playlists" label="Playlists" />
        <NavItem to="/dashboard" label="Dashboard" />
        <div className="nav-divider" />

        <p className="sidebar-section-label">Your Library</p>

        <NavItem to="/favorites" label="Favorites" />
        <NavItem to="/download" label="Download" />
        <NavItem to="/playlists" label="Playlists" />

        <div className="nav-divider" />
      </nav>
      <div className="sidebar-footer">{user ? <AccountPopover /> : <AuthDialog />}</div>
    </aside>
  );
}
