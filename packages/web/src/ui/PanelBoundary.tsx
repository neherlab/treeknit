import { type ReactNode, useCallback, useMemo } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import RetryIcon from "~icons/lucide/rotate-ccw";

import { Button } from "./Button";
import { InlineNotice } from "./InlineNotice";

export function PanelBoundary({ title, children }: PanelBoundaryProps) {
  const fallback = useCallback((props: FallbackProps) => <PanelFailed title={title} {...props} />, [title]);

  return <ErrorBoundary fallbackRender={fallback}>{children}</ErrorBoundary>;
}

export interface PanelBoundaryProps {
  title: string;
  children: ReactNode;
}

function PanelFailed({ title, error, resetErrorBoundary }: FallbackProps & { title: string }) {
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
    <div className="p-3">
      <InlineNotice tone="danger" title={title} action={action}>
        <p>{getErrorMessage(error) ?? String(error)}</p>
        <p>The other views are still available.</p>
      </InlineNotice>
    </div>
  );
}
