import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import { getErrorMessage } from "react-error-boundary";

import { useWorkspaceAvailability } from "./availability";
import {
  type PairRef,
  resolveWorkspaceSearch,
  type WorkspaceSearch,
  withSelectionPair,
  type WrittenSearch,
} from "./search";

const workspaceRoute = getRouteApi("/");

export function useWorkspaceSearch(): WorkspaceSearchState {
  const search = workspaceRoute.useSearch();
  const navigate = workspaceRoute.useNavigate();
  const availability = useWorkspaceAvailability();
  const resolved = useMemo(() => resolveWorkspaceSearch(search, availability), [search, availability]);
  const [failure, setFailure] = useState<Error | null>(null);
  const { pairLabels } = availability;
  const resolvedPair = resolved.pair;

  const update = useCallback(
    (change: (written: WrittenSearch) => WrittenSearch, options?: SearchUpdateOptions) => {
      navigate({
        search: (written: WrittenSearch) => withSelectionPair(written, change(written), pairLabels, resolvedPair),
        replace: options?.replace ?? false,
      }).catch((cause: unknown) => {
        setFailure(new Error(`The view could not be updated: ${getErrorMessage(cause) ?? String(cause)}`, { cause }));
      });
    },
    [navigate, pairLabels, resolvedPair],
  );

  if (failure !== null) {
    throw failure;
  }

  return { search: resolved, update, pairLabels };
}

export interface WorkspaceSearchState {
  search: WorkspaceSearch;
  update: (change: (written: WrittenSearch) => WrittenSearch, options?: SearchUpdateOptions) => void;
  pairLabels: readonly PairRef[];
}

export interface SearchUpdateOptions {
  replace?: boolean;
}
