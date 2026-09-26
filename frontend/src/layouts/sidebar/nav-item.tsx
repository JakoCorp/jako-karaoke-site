import { NavLink } from "react-router";

interface NavItemProps {
  to: string;
  label: string;
  onClick?: () => void;
}

export function NavItem({ to, label, onClick }: NavItemProps) {
  return (
    <NavLink
      to={to}
      onClick={onClick}
      className={({ isActive }: { isActive: boolean }) =>
        isActive ? "nav-item nav-item--active" : "nav-item"
      }
    >
      {label}
    </NavLink>
  );
}
