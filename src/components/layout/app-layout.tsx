import { LayoutDashboard, Settings } from "lucide-react";
import { NavLink, Outlet } from "react-router";

import { cn } from "../../lib/utils/cn";

const navigationItems = [
  { label: "Layouts", path: "/layouts", icon: LayoutDashboard },
  { label: "Réglages", path: "/settings", icon: Settings },
];

export function AppLayout() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <a
        className="sr-only focus:not-sr-only focus:absolute focus:left-6 focus:top-4 focus:z-50 focus:rounded-md focus:bg-surface focus:px-3 focus:py-2 focus:text-sm focus:font-medium"
        href="#main-content"
      >
        Aller au contenu
      </a>
      <header className="border-b border-border bg-surface">
        <div className="mx-auto flex h-16 max-w-6xl items-center px-6">
          <span className="text-base font-semibold tracking-tight">Layout Manager 2</span>
        </div>
      </header>
      <div className="mx-auto grid max-w-6xl grid-cols-[12rem_1fr] gap-8 px-6 py-8">
        <nav aria-label="Navigation principale" className="flex flex-col gap-1">
          {navigationItems.map(({ icon: Icon, label, path }) => (
            <NavLink
              className={({ isActive }) =>
                cn(
                  "flex h-10 items-center gap-3 rounded-md px-3 text-sm font-medium text-muted-foreground hover:bg-muted hover:text-foreground",
                  isActive && "bg-muted text-foreground",
                )
              }
              key={path}
              to={path}
            >
              <Icon aria-hidden="true" className="size-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        <main className="min-w-0" id="main-content" tabIndex={-1}>
          <Outlet />
        </main>
      </div>
    </div>
  );
}
