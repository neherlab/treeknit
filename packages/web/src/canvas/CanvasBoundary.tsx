import { QueryErrorResetBoundary, useQueryErrorResetBoundary } from "@tanstack/react-query";
import { type ReactNode, Suspense, useCallback, useMemo } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { Button } from "../ui/Button";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { DrawingCodeError } from "./lazyCanvas";

const LOADING = (
  <div className="flex h-full items-center justify-center p-6">
    <ProgressBar label="Loading the drawing" isIndeterminate className="w-64" />
  </div>
);

export interface CanvasBoundaryProps {
  resultKey: string;
  onReset: () => void;
  children: ReactNode;
}

export function CanvasBoundary(props: CanvasBoundaryProps) {
  return (
    <QueryErrorResetBoundary>
      <CanvasErrorBoundary {...props} />
    </QueryErrorResetBoundary>
  );
}

function CanvasErrorBoundary({ resultKey, onReset, children }: CanvasBoundaryProps) {
  const resetKeys = useMemo(() => [resultKey], [resultKey]);
  const { reset } = useQueryErrorResetBoundary();

  const resetAll = useCallback(() => {
    reset();
    onReset();
  }, [reset, onReset]);

  return (
    <ErrorBoundary FallbackComponent={DrawingFailed} resetKeys={resetKeys} onReset={resetAll}>
      <Suspense fallback={LOADING}>{children}</Suspense>
    </ErrorBoundary>
  );
}

function DrawingFailed({ error, resetErrorBoundary }: FallbackProps) {
  const retry = useCallback(() => {
    resetErrorBoundary();
  }, [resetErrorBoundary]);

  const action = useMemo(
    () => (
      <Button variant="secondary" size="sm" icon={RetryIcon} onPress={retry}>
        Retry
      </Button>
    ),
    [retry],
  );

  return (
    <InlineNotice tone="danger" title="The drawing could not be shown" action={action}>
      <p>{getErrorMessage(error) ?? String(error)}</p>
      {error instanceof DrawingCodeError ? (
        <p>Check the connection and try again. If that fails again, reload the page.</p>
      ) : null}
      <p>The tables and files are still available.</p>
    </InlineNotice>
  );
}
