import { type ReactNode, Suspense, useCallback, useMemo } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { useAuspiceView } from "../analysis/queries";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { PairSelect, ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
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
  const { search, select, clear, choosePair, chooseVersion, chooseScale } = useDrawingSearch();
  const { pair, version, x } = search;
  const query = useAuspiceView(result.sessionId, pair, version, x);
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
      query={query}
      loading="Loading the Auspice view"
      errorTitle="The Auspice view could not be loaded"
      onEscape={clear}
    >
      {(datasets) =>
        labels === undefined ? null : (
          <AuspiceBoundary
            resultKey={`${String(result.sessionId)}:${String(pair)}:${version}:${x}`}
            onReset={reloadAuspiceView}
          >
            <AuspiceView datasets={datasets} labels={labels} onSelect={select} />
          </AuspiceBoundary>
        )
      }
    </DrawingPanel>
  );
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
