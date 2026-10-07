import { QueryErrorResetBoundary, useQueryErrorResetBoundary } from "@tanstack/react-query";
import { type ReactNode, useCallback, useMemo } from "react";

import { PanelBoundary } from "../ui/PanelBoundary";
import { DrawingCodeError } from "./lazyCanvas";

const DRAWING_NOTES: readonly string[] = ["The tables and files are still available."];

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
    <PanelBoundary
      title="The drawing could not be shown"
      notes={DRAWING_NOTES}
      hint={drawingHint}
      loading="Loading the drawing"
      resetKeys={resetKeys}
      onReset={resetAll}
    >
      {children}
    </PanelBoundary>
  );
}

function drawingHint(error: Error): string | undefined {
  return error instanceof DrawingCodeError
    ? "Check the connection and try again. If that fails again, reload the page."
    : undefined;
}
