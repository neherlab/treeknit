import type { ArgView, PairView } from "@neherlab/treeknit-wasm";
import { notifyManager, type QueryClient, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo, useSyncExternalStore } from "react";

import { analysisKeys } from "../analysis/queries";
import { internalNodeIndex } from "../drawing/trees";
import { useWorkspace } from "./context";
import { NO_WORKSPACE, type NodeRef, type PairRef, resolvePair, type WorkspaceAvailability } from "./search";
import type { RunResult } from "./store";

const workspaceRoute = getRouteApi("/");

const pairLabelCache = new WeakMap<RunResult, readonly PairRef[]>();

export interface LoadedViews {
  pairView: (pair: number) => PairView | undefined;
  argView: () => ArgView | null | undefined;
}

export function useWorkspaceAvailability(): WorkspaceAvailability {
  const treeCount = useWorkspace((state) => state.trees.length);
  const result = useWorkspace((state) => state.result);
  const sessionId = result?.sessionId;

  const {
    pair: searchPair,
    version,
    scale,
  } = workspaceRoute.useSearch({
    select: (search) => ({ pair: search.pair, version: search.version, scale: search.scale }),
    structuralSharing: true,
  });

  const pair = resolvePair(searchPair, pairLabels(result));

  const loadedPair = useCachedData((queryClient) =>
    sessionId === undefined
      ? undefined
      : queryClient.getQueryData<PairView>(analysisKeys.pairView(sessionId, pair, version, scale)),
  );

  const loadedArg = useCachedData((queryClient) =>
    sessionId === undefined
      ? undefined
      : queryClient.getQueryData<ArgView | null>(analysisKeys.argView(sessionId, scale)),
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
    pairLabels: pairLabels(result),
    mccExists: (pair, mcc) => Number.isInteger(mcc) && mcc >= 0 && mcc < (pairs[pair]?.mccCount ?? 0),
    leafExists: (pair, leaf) => leavesOf(pair).has(leaf),
    nodeExists: (pair, node) => pair < pairs.length && nodeInLoadedView(pair, node, views),
  };
}

export function pairLabels(result: RunResult | null): readonly PairRef[] {
  if (result === null) {
    return NO_WORKSPACE.pairLabels;
  }

  const cached = pairLabelCache.get(result);

  if (cached !== undefined) {
    return cached;
  }

  const labels = result.summary.pairs.map(({ labels: [first, second] }): PairRef => [first, second]);

  pairLabelCache.set(result, labels);

  return labels;
}

function nodeInLoadedView(pair: number, node: NodeRef, views: LoadedViews): boolean {
  if (node.side === "arg") {
    const arg = views.argView();

    return arg === undefined || (arg !== null && arg.nodes.some(({ label }) => label === node.name));
  }

  const view = views.pairView(pair);

  return view === undefined || internalNodeIndex(view[node.side], node.name) !== undefined;
}

export function subscribeToQueryCache(queryClient: QueryClient, onChange: () => void): () => void {
  return queryClient.getQueryCache().subscribe(notifyManager.batchCalls(onChange));
}

function useCachedData<T>(read: (queryClient: QueryClient) => T | undefined): T | undefined {
  const queryClient = useQueryClient();

  const subscribe = useCallback((onChange: () => void) => subscribeToQueryCache(queryClient, onChange), [queryClient]);

  return useSyncExternalStore(subscribe, () => read(queryClient));
}
