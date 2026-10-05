import { Outlet } from "@tanstack/react-router";

import { Header } from "./Header";

export function AppShell() {
  return (
    <div className="bg-ground text-ink flex h-dvh min-h-0 flex-col">
      <Header />
      <Outlet />
    </div>
  );
}
