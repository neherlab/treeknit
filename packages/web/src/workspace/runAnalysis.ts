import type { AnalysisClient, RunOutcome } from "../analysis/client";
import { selectRequest, selectRunning, type WorkspaceStore } from "./store";

export async function runAnalysis(
  client: Pick<AnalysisClient, "startRun">,
  store: WorkspaceStore,
): Promise<FinishedRun | null> {
  const state = store.getState();

  if (selectRunning(state)) {
    return null;
  }

  const request = selectRequest(state);

  const handle = client.startRun(request, (progress) => {
    store.getState().runProgressed(handle.runId, progress);
  });

  store.getState().runStarted(handle.runId, request);

  const outcome = await handle.outcome;

  store.getState().runFinished(handle.runId, outcome);

  return { outcome, storedSessionId: store.getState().result?.sessionId };
}

export interface FinishedRun {
  outcome: RunOutcome;
  storedSessionId: number | undefined;
}
