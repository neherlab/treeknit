import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useWorkspaceAvailability } from "./availability";
import { resolveWorkspaceSearch, type WorkspaceSearch } from "./search";

const workspaceRoute = getRouteApi("/");

export function useWorkspaceSearch(): WorkspaceSearchState {
  const search = workspaceRoute.useSearch();
  const navigate = workspaceRoute.useNavigate();
  const availability = useWorkspaceAvailability();
  const resolved = useMemo(() => resolveWorkspaceSearch(search, availability), [search, availability]);

  const update = useCallback(
    (change: (current: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) =>
      navigate({
        search: change(resolved),
        replace: options?.replace ?? false,
      }),
    [navigate, resolved],
  );

  return { search: resolved, update };
}

export interface WorkspaceSearchState {
  search: WorkspaceSearch;
  update: (change: (current: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) => Promise<void>;
}

export interface SearchUpdateOptions {
  replace?: boolean;
}
