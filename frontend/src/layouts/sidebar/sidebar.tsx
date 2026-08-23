import { NavItem } from "./nav-item";

export function Sidebar() {
  return (
    <aside className="flex h-full w-80 shrink-0 flex-col bg-neutral-700 text-white">
      <div className="px-4 py-5">
        <span className="text-lg font-bold text-neutral-100"> Karaoke Player</span>
      </div>
      <nav className="flex-1 space-y-1 overflow-y-auto px-2 pb-4">
        <NavItem to="/" label="Home" />
        <NavItem to="/search" label="Search" />
        <NavItem to="/randomsong" label="Random Songs" />
        <NavItem to="/explore" label="Explore" />
        <NavItem to="/artists" label="Artists" />
        <NavItem to="/playlists" label="Playlists" />
        <NavItem to="/dashboard" label="Dashboard" />
        <div className="my-5 border-t border-white/50" />

        <p className="text-s px-3 pb-2 font-semibold tracking-wider text-neutral-100 uppercase">
          Your Library
        </p>

        <NavItem to="/favorites" label="Favorites" />
        <NavItem to="/download" label="Download" />
        <NavItem to="/playlists" label="Playlists" />

        <div className="my-5 border-t border-white/50" />
      </nav>
    </aside>
  );
}
