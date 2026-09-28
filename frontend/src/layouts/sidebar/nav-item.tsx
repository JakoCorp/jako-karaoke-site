import type { Icon } from "@phosphor-icons/react";
import { NavLink } from "react-router";

interface NavItemProps {
  to: string;
  label: string;
  icon?: Icon;
  onClick?: () => void;
}

export function NavItem({ to, label, icon: NavIcon, onClick }: NavItemProps) {
  return (
    <NavLink
      to={to}
      onClick={onClick}
      className={({ isActive }: { isActive: boolean }) =>
        isActive ? "nav-item nav-item--active" : "nav-item"
      }
    >
      {NavIcon && <NavIcon size={16} />}
      {label}
    </NavLink>
  );
}
