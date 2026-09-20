import { XIcon } from "@phosphor-icons/react";

import logoUrl from "@/assets/Baji.factions.Industry.svg";
import { AuthDialog } from "@/features/auth";
import { useAuthStore } from "@/store/auth";

import { NavItem } from "./nav-item";
import { UserMenu } from "./user-menu";

type SidebarProps = {
  open: boolean;
  onClose: () => void;
};

export function Sidebar({ open, onClose }: SidebarProps) {
  const user = useAuthStore((state) => state.user);

  return (
    <aside
      className={`fixed inset-0 z-50 sidebar flex h-full w-full flex-col transition-transform duration-300 ease-in-out md:static md:z-auto md:flex md:w-60 md:translate-y-0 ${open ? "translate-y-0" : "-translate-y-full"}`}
    >
      <div className="sidebar-header">
        <img src={logoUrl} alt="" aria-hidden className="sidebar-logo" />
        <span className="sidebar-brand">Karaoke Player</span>

        <button
          type="button"
          onClick={onClose}
          className="ml-auto cursor-pointer md:hidden"
          aria-label="Close Sidebar"
        >
          <XIcon size={28} />
        </button>
      </div>
      <nav className="sidebar-nav">
        <NavItem to="/" label="Home" onClick={onClose} />
        <NavItem to="/search" label="Search" onClick={onClose} />
        <NavItem to="/random-song" label="Random Songs" onClick={onClose} />
        <NavItem to="/public-playlists" label="Public Playlists" onClick={onClose} />
        <NavItem to="/artists" label="Artists" onClick={onClose} />
        <NavItem to="/about" label="About" onClick={onClose} />
        <div className="nav-divider" />

        <p className="sidebar-section-label">Your Library</p>

        <NavItem to="/favorites" label="Favorites" onClick={onClose} />
        <NavItem to="/my-playlists" label="My Playlists" onClick={onClose} />
        <NavItem to="/local" label="Local" onClick={onClose} />

        <div className="nav-divider" />
      </nav>
      <div className="sidebar-footer">{user ? <UserMenu /> : <AuthDialog />}</div>
    </aside>
  );
}
