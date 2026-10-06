import type { AuspicePair } from "@neherlab/treeknit-wasm";
import { type ReactNode, Suspense, useCallback, useMemo, useState } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { useAuspiceFiles, useAuspiceView } from "../analysis/queries";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { LeafSearch } from "../drawing/LeafSearch";
import { shownScaleNotice } from "../drawing/scale";
import { useDrawingSearch } from "../drawing/useDrawingSearch";
import { Button } from "../ui/Button";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { useWorkspace } from "../workspace/context";
import { selectPair } from "../workspace/search";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { leafNames, shownTreeLabels } from "./datasets";
import { parseAuspiceQuery, withAuspiceQuery } from "./query";
import type { ShownTrees } from "./strip";
import { TreeStrip } from "./TreeStrip";

const AUSPICE_VIEW = lazyCanvas(async () => import("./AuspiceView"));

const LOADING = (
  <div className="flex h-full items-center justify-center p-6">
    <ProgressBar label="Loading Auspice" isIndeterminate className="w-64" />
  </div>
);

export function AuspicePanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Auspice result={result} />;
}

function Auspice({ result }: { result: RunResult }) {
  const [AuspiceView, reloadAuspiceView] = useLazyCanvas(AUSPICE_VIEW);
  const { search, selection, select, clearOnEscape, chooseVersion, chooseScale } = useDrawingSearch();
  const { update } = useWorkspaceSearch();
  const { pair, version, x, trees } = search;
  const query = useAuspiceView(result.sessionId, pair, version, x);
  const files = useAuspiceFiles(result.sessionId, pair, version, x, trees).data;
  const pairs = result.summary.pairs;
  const labels = pairs[pair]?.labels;
  const treeLabels = useMemo(() => result.request.trees.map(({ label }) => label), [result.request.trees]);
  const shown = useMemo(() => ({ pair, trees }), [pair, trees]);
  const shownLabels = useMemo(() => shownTreeLabels(labels, trees), [labels, trees]);
  const urlQuery = useMemo(() => parseAuspiceQuery(search.auspice), [search.auspice]);
  const names = useMemo(() => (query.data === undefined ? [] : leafNames(query.data, trees)), [query.data, trees]);

  const chooseShown = useCallback(
    (next: ShownTrees) => {
      update((written) => ({
        ...(next.pair === written.pair ? written : selectPair(written, next.pair)),
        trees: next.trees,
      }));
    },
    [update],
  );

  const writeQuery = useCallback(
    (text: string) => {
      update((written) => withAuspiceQuery(written, text), { replace: true });
    },
    [update],
  );

  const findLeaf = useCallback(
    (name: string) => {
      select({ leaf: name });
    },
    [select],
  );

  const toolbar = (
    <>
      <TreeStrip labels={treeLabels} pairs={pairs} shown={shown} onChange={chooseShown} />
      <VersionToggle value={version} onChange={chooseVersion} />
      <ScaleToggle value={x} onChange={chooseScale} />
      <LeafSearch names={names} onSelect={findLeaf} />
    </>
  );

  return (
    <DrawingPanel
      toolbar={toolbar}
      notice={shownScaleNotice(x, query.data?.scale, "pair")}
      query={query}
      loading="Loading the Auspice view"
      errorTitle="The Auspice view could not be loaded"
      onEscape={clearOnEscape}
    >
      {(datasets) =>
        labels === undefined ? null : (
          <AuspiceBoundary
            resultKey={`${String(result.sessionId)}:${String(pair)}:${version}:${x}`}
            onReset={reloadAuspiceView}
          >
            <KeyedAuspiceView datasets={datasets}>
              {(key) => (
                <AuspiceView
                  key={`${String(key)}:${trees}`}
                  datasets={datasets}
                  labels={labels}
                  trees={trees}
                  query={urlQuery}
                  selection={selection}
                  pair={pair}
                  onSelect={select}
                  onQuery={writeQuery}
                  files={files}
                  treeLabels={shownLabels}
                  axisTitle={datasets.axis_title}
                />
              )}
            </KeyedAuspiceView>
          </AuspiceBoundary>
        )
      }
    </DrawingPanel>
  );
}

function KeyedAuspiceView({ datasets, children }: { datasets: AuspicePair; children: (key: number) => ReactNode }) {
  return children(useDatasetsGeneration(datasets));
}

function useDatasetsGeneration(datasets: AuspicePair): number {
  const [shown, setShown] = useState({ datasets, generation: 0 });

  if (shown.datasets !== datasets) {
    const next = { datasets, generation: shown.generation + 1 };

    setShown(next);

    return next.generation;
  }

  return shown.generation;
}

function AuspiceBoundary({ resultKey, onReset, children }: AuspiceBoundaryProps) {
  const resetKeys = useMemo(() => [resultKey], [resultKey]);

  return (
    <ErrorBoundary FallbackComponent={AuspiceFailed} resetKeys={resetKeys} onReset={onReset}>
      <Suspense fallback={LOADING}>{children}</Suspense>
    </ErrorBoundary>
  );
}

interface AuspiceBoundaryProps {
  resultKey: string;
  onReset: () => void;
  children: ReactNode;
}

function AuspiceFailed({ error, resetErrorBoundary }: FallbackProps) {
  const retry = useCallback(() => {
    resetErrorBoundary();
  }, [resetErrorBoundary]);

  const action = useMemo(
    () => (
      <Button variant="secondary" size="sm" icon={RetryIcon} onPress={retry}>
        Try again
      </Button>
    ),
    [retry],
  );

  return (
    <div className="p-3">
      <InlineNotice tone="danger" title="Auspice cannot draw these trees." action={action}>
        <p>{getErrorMessage(error) ?? String(error)}</p>
        <p>The tanglegram, the tables, and the files are still available.</p>
      </InlineNotice>
    </div>
  );
}
