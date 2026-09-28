import {
  GlobeIcon,
  HardDriveIcon,
  HeartIcon,
  HouseIcon,
  InfoIcon,
  MagnifyingGlassIcon,
  MicrophoneIcon,
  QueueIcon,
  ShuffleIcon,
  XIcon,
} from "@phosphor-icons/react";

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
    <aside className="sidebar" data-open={open}>
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
        <NavItem to="/" label="Home" icon={HouseIcon} onClick={onClose} />
        <NavItem to="/search" label="Search" icon={MagnifyingGlassIcon} onClick={onClose} />
        <NavItem to="/random-song" label="Random Songs" icon={ShuffleIcon} onClick={onClose} />
        <NavItem
          to="/public-playlists"
          label="Public Playlists"
          icon={GlobeIcon}
          onClick={onClose}
        />
        <NavItem to="/artists" label="Artists" icon={MicrophoneIcon} onClick={onClose} />
        <NavItem to="/about" label="About" icon={InfoIcon} onClick={onClose} />
        <div className="nav-divider" />

        <p className="sidebar-section-label">Your Library</p>

        <NavItem to="/favorites" label="Favorites" icon={HeartIcon} onClick={onClose} />
        <NavItem to="/my-playlists" label="My Playlists" icon={QueueIcon} onClick={onClose} />
        <NavItem to="/local" label="Local" icon={HardDriveIcon} onClick={onClose} />

        <div className="nav-divider" />
      </nav>
      <div className="sidebar-footer">{user ? <UserMenu /> : <AuthDialog />}</div>
    </aside>
  );
}
