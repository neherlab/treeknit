import type { AuspicePair } from "@neherlab/treeknit-wasm";
import { type ReactNode, Suspense, useCallback, useMemo, useState } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { useAuspiceFiles, useAuspiceView } from "../analysis/queries";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { PairSelect, ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { shownScaleNotice } from "../drawing/scale";
import { useDrawingSearch } from "../drawing/useDrawingSearch";
import { Button } from "../ui/Button";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";

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
  const { search, select, clearOnEscape, choosePair, chooseVersion, chooseScale } = useDrawingSearch();
  const { pair, version, x } = search;
  const query = useAuspiceView(result.sessionId, pair, version, x);
  const files = useAuspiceFiles(result.sessionId, pair, version, x, "both").data;
  const pairs = result.summary.pairs;
  const labels = pairs[pair]?.labels;

  const toolbar = (
    <>
      <PairSelect pairs={pairs} value={pair} onChange={choosePair} />
      <VersionToggle value={version} onChange={chooseVersion} />
      <ScaleToggle value={x} onChange={chooseScale} />
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
                  key={key}
                  datasets={datasets}
                  labels={labels}
                  onSelect={select}
                  files={files}
                  treeLabels={labels}
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
