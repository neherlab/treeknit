import { type ReactNode, Suspense, useCallback, useMemo } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import { doNothing } from "remeda";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { Button } from "./Button";
import { ErrorNotice } from "./ErrorNotice";
import { Loading } from "./Loading";

const OTHER_VIEWS_AVAILABLE: readonly string[] = ["The other views are still available."];

const NO_RESET_KEYS: unknown[] = [];

const NO_RESET = doNothing();

export function PanelBoundary({
  title,
  notes = OTHER_VIEWS_AVAILABLE,
  hint = noHint,
  loading,
  resetKeys = NO_RESET_KEYS,
  onReset = NO_RESET,
  children,
}: PanelBoundaryProps) {
  const fallback = useCallback(
    (props: FallbackProps) => <PanelFailed title={title} notes={notes} hint={hint} {...props} />,
    [title, notes, hint],
  );

  const loadingFallback = useMemo(() => (loading === undefined ? null : <Loading label={loading} />), [loading]);

  return (
    <ErrorBoundary fallbackRender={fallback} resetKeys={resetKeys} onReset={onReset}>
      {loadingFallback === null ? children : <Suspense fallback={loadingFallback}>{children}</Suspense>}
    </ErrorBoundary>
  );
}

export interface PanelBoundaryProps {
  title: string;
  notes?: readonly string[];
  hint?: (error: Error) => string | undefined;
  loading?: string;
  resetKeys?: unknown[];
  onReset?: () => void;
  children: ReactNode;
}

function PanelFailed({
  title,
  notes,
  hint,
  error,
  resetErrorBoundary,
}: FallbackProps & Required<Pick<PanelBoundaryProps, "title" | "notes" | "hint">>) {
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

  const errorHint = error instanceof Error ? hint(error) : undefined;

  const details = useMemo(
    () => [getErrorMessage(error) ?? String(error), ...(errorHint === undefined ? [] : [errorHint]), ...notes],
    [error, errorHint, notes],
  );

  return (
    <div className="p-3">
      <ErrorNotice title={title} details={details} action={action} />
    </div>
  );
}

function noHint(): undefined {
  return undefined;
}
