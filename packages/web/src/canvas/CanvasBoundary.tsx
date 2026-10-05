import {
  type ComponentType,
  lazy,
  type LazyExoticComponent,
  type ReactNode,
  Suspense,
  useCallback,
  useMemo,
  useState,
} from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { Button } from "../ui/Button";
import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";

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

export function CanvasBoundary({ resultKey, onReset, children }: CanvasBoundaryProps) {
  const resetKeys = useMemo(() => [resultKey], [resultKey]);

  return (
    <ErrorBoundary FallbackComponent={DrawingFailed} resetKeys={resetKeys} onReset={onReset}>
      <Suspense fallback={LOADING}>{children}</Suspense>
    </ErrorBoundary>
  );
}

export function useLazyCanvas<P extends object>(
  load: () => Promise<{ default: ComponentType<P> }>,
): readonly [LazyExoticComponent<ComponentType<P>>, () => void] {
  const [Canvas, setCanvas] = useState(() => lazy(load));

  const reload = useCallback(() => {
    setCanvas(() => lazy(load));
  }, [load]);

  return [Canvas, reload];
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
      <p>The tables and files are still available.</p>
    </InlineNotice>
  );
}
