import { Outlet } from "@tanstack/react-router";

export function AppShell() {
  return (
    <div className="bg-ground text-ink flex h-dvh min-h-0 flex-col">
      <Outlet />
    </div>
  );
}
