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
        `block rounded-md px-3 py-2 text-sm transition-colors ${
          isActive
            ? "bg-primary text-primary-foreground"
            : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        }`
      }
    >
      {label}
    </NavLink>
  );
}
