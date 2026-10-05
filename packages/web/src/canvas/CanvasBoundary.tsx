import { type ReactNode, Suspense } from "react";
import { ErrorBoundary, type FallbackProps } from "react-error-boundary";

import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";

const LOADING = (
  <div className="flex h-full items-center justify-center p-6">
    <ProgressBar label="Loading the drawing" isIndeterminate className="w-64" />
  </div>
);

export interface CanvasBoundaryProps {
  children: ReactNode;
}

export function CanvasBoundary({ children }: CanvasBoundaryProps) {
  return (
    <ErrorBoundary FallbackComponent={DrawingFailed}>
      <Suspense fallback={LOADING}>{children}</Suspense>
    </ErrorBoundary>
  );
}

function DrawingFailed({ error }: FallbackProps) {
  const message = error instanceof Error ? error.message : String(error);

  return (
    <InlineNotice tone="danger" title="The drawing could not be shown">
      {message}. The tables and files are still available.
    </InlineNotice>
  );
}
