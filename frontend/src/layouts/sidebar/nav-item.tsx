import { NavLink } from "react-router";

interface NavItemProps {
  to: string;
  label: string;
}

export function NavItem({ to, label }: NavItemProps) {
  return (
    <NavLink
      to={to}
      className={({ isActive }: { isActive: boolean }) =>
        isActive ? "nav-item nav-item--active" : "nav-item"
      }
    >
      {label}
    </NavLink>
  );
}
