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
    (change: (written: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) => {
      navigate({ search: change, replace: options?.replace ?? false }).catch((error) => {
        console.error("The workspace search could not be updated:", error);
      });
    },
    [navigate],
  );

  return { search: resolved, update };
}

export interface WorkspaceSearchState {
  search: WorkspaceSearch;
  update: (change: (written: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) => void;
}

export interface SearchUpdateOptions {
  replace?: boolean;
}
