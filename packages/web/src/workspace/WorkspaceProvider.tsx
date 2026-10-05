import { type ReactNode, Suspense, use } from "react";
import { ErrorBoundary, type FallbackProps } from "react-error-boundary";

import { useDelayedIndicator } from "../indicator/useDelayedIndicator";
import { InlineNotice } from "../ui/InlineNotice";
import { WorkspaceContext } from "./context";
import type { WorkspaceRuntime } from "./runtime";

const STARTING = <Starting />;

export function WorkspaceProvider({ runtime, children }: WorkspaceProviderProps) {
  return (
    <ErrorBoundary FallbackComponent={StartFailure}>
      <Suspense fallback={STARTING}>
        <ReadyWorkspace runtime={runtime}>{children}</ReadyWorkspace>
      </Suspense>
    </ErrorBoundary>
  );
}

export interface WorkspaceProviderProps {
  runtime: Promise<WorkspaceRuntime>;
  children: ReactNode;
}

function ReadyWorkspace({ runtime, children }: WorkspaceProviderProps) {
  const ready = use(runtime);

  return <WorkspaceContext value={ready}>{children}</WorkspaceContext>;
}

function Starting() {
  const visible = useDelayedIndicator(true);

  return visible ? <output className="text-ink-muted block p-6 text-sm">Starting TreeKnit</output> : null;
}

function StartFailure({ error }: FallbackProps) {
  return (
    <div className="mx-auto max-w-xl p-6">
      <InlineNotice tone="danger" title="TreeKnit could not start in this browser.">
        <p>{error instanceof Error ? error.message : String(error)}</p>
        <p>Use a current version of Chrome, Edge, Firefox, or Safari.</p>
      </InlineNotice>
    </div>
  );
}
