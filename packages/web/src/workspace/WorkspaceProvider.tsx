import { type ReactNode, Suspense, use } from "react";
import { ErrorBoundary, type FallbackProps, getErrorMessage } from "react-error-boundary";
import { useSpinDelay } from "spin-delay";

import { INDICATOR_SPIN_DELAY } from "../indicator/timing";
import { failureNotice } from "../run/failure";
import { SettingsFormProvider } from "../settings/SettingsFormProvider";
import { ErrorNotice } from "../ui/ErrorNotice";
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

  return (
    <WorkspaceContext value={ready}>
      <SettingsFormProvider>{children}</SettingsFormProvider>
    </WorkspaceContext>
  );
}

function Starting() {
  const visible = useSpinDelay(true, INDICATOR_SPIN_DELAY);

  return visible ? <output className="text-ink-muted block p-6 text-sm">Starting TreeKnit</output> : null;
}

function StartFailure({ error }: FallbackProps) {
  const notice = failureNotice("start", getErrorMessage(error) ?? String(error));

  return (
    <div className="mx-auto max-w-xl p-6">
      <ErrorNotice title={notice.title} details={notice.details} />
    </div>
  );
}
