import type { ArgView, PairView } from "@neherlab/treeknit-wasm";
import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo, useSyncExternalStore } from "react";

import { analysisKeys } from "../analysis/queries";
import { useWorkspace } from "./context";
import { NO_WORKSPACE, type NodeRef, type WorkspaceAvailability } from "./search";
import type { RunResult } from "./store";

const workspaceRoute = getRouteApi("/");

export interface LoadedViews {
  pairView: (pair: number) => PairView | undefined;
  argView: () => ArgView | null | undefined;
}

export function useWorkspaceAvailability(): WorkspaceAvailability {
  const treeCount = useWorkspace((state) => state.trees.length);
  const result = useWorkspace((state) => state.result);
  const sessionId = result?.sessionId;

  const { pair, version, x } = workspaceRoute.useSearch({
    select: (search) => ({ pair: search.pair, version: search.version, x: search.x }),
    structuralSharing: true,
  });

  const loadedPair = useCachedData((queryClient) =>
    sessionId === undefined
      ? undefined
      : queryClient.getQueryData<PairView>(analysisKeys.pairView(sessionId, pair, version, x)),
  );

  const loadedArg = useCachedData((queryClient) =>
    sessionId === undefined ? undefined : queryClient.getQueryData<ArgView | null>(analysisKeys.argView(sessionId, x)),
  );

  return useMemo(
    () =>
      workspaceAvailability(treeCount, result, {
        pairView: (viewPair) => (viewPair === pair ? loadedPair : undefined),
        argView: () => loadedArg,
      }),
    [treeCount, result, pair, loadedPair, loadedArg],
  );
}

export function workspaceAvailability(
  treeCount: number,
  result: RunResult | null,
  views: LoadedViews,
): WorkspaceAvailability {
  if (result === null) {
    return { ...NO_WORKSPACE, treeCount };
  }

  const pairs = result.summary.pairs;
  const leafSets = new Map<number, ReadonlySet<string>>();

  const leavesOf = (pair: number): ReadonlySet<string> => {
    const cached = leafSets.get(pair);

    if (cached !== undefined) {
      return cached;
    }

    const leaves = new Set(pairs[pair]?.mccs.flat() ?? []);

    leafSets.set(pair, leaves);

    return leaves;
  };

  return {
    hasResult: true,
    treeCount,
    resultTreeCount: result.request.trees.length,
    pairCount: pairs.length,
    mccExists: (pair, mcc) => Number.isInteger(mcc) && mcc >= 0 && mcc < (pairs[pair]?.mccCount ?? 0),
    leafExists: (pair, leaf) => leavesOf(pair).has(leaf),
    nodeExists: (pair, node) => pair < pairs.length && nodeInLoadedView(pair, node, views),
  };
}

function nodeInLoadedView(pair: number, node: NodeRef, views: LoadedViews): boolean {
  if (node.side === "arg") {
    const arg = views.argView();

    return arg === undefined || (arg !== null && arg.nodes.some(({ label }) => label === node.name));
  }

  const view = views.pairView(pair);

  return view === undefined || view[node.side].nodes.some(({ name }) => name === node.name);
}

function useCachedData<T>(read: (queryClient: QueryClient) => T | undefined): T | undefined {
  const queryClient = useQueryClient();

  const subscribe = useCallback(
    (onChange: () => void) => queryClient.getQueryCache().subscribe(onChange),
    [queryClient],
  );

  return useSyncExternalStore(subscribe, () => read(queryClient));
}
