import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import { getErrorMessage } from "react-error-boundary";

import { useWorkspaceAvailability } from "./availability";
import { resolveWorkspaceSearch, type WorkspaceSearch, withSelectionPair } from "./search";

const workspaceRoute = getRouteApi("/");

export function useWorkspaceSearch(): WorkspaceSearchState {
  const search = workspaceRoute.useSearch();
  const navigate = workspaceRoute.useNavigate();
  const availability = useWorkspaceAvailability();
  const resolved = useMemo(() => resolveWorkspaceSearch(search, availability), [search, availability]);
  const [failure, setFailure] = useState<Error | null>(null);
  const resolvedPair = resolved.pair;

  const update = useCallback(
    (change: (written: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) => {
      navigate({
        search: (written: WorkspaceSearch) => withSelectionPair(written, change(written), resolvedPair),
        replace: options?.replace ?? false,
      }).catch((cause: unknown) => {
        setFailure(new Error(`The view could not be updated: ${getErrorMessage(cause) ?? String(cause)}`, { cause }));
      });
    },
    [navigate, resolvedPair],
  );

  if (failure !== null) {
    throw failure;
  }

  return { search: resolved, update };
}

export interface WorkspaceSearchState {
  search: WorkspaceSearch;
  update: (change: (written: WorkspaceSearch) => WorkspaceSearch, options?: SearchUpdateOptions) => void;
}

export interface SearchUpdateOptions {
  replace?: boolean;
}
