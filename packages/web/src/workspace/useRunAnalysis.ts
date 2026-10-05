import { useMutation } from "@tanstack/react-query";
import { DateTime } from "luxon";
import { useCallback } from "react";

import type { RunHandle, RunOutcome } from "../analysis/client";
import { useAnalysisClient } from "../analysis/context";
import { useWorkspaceStore } from "./context";
import { searchAfterRun } from "./search";
import { selectRequest, selectRunning } from "./store";
import { useWorkspaceSearch } from "./useWorkspaceSearch";

export function useRunAnalysis(): RunControl {
  const client = useAnalysisClient();
  const store = useWorkspaceStore();
  const { update } = useWorkspaceSearch();

  const { mutate } = useMutation<RunOutcome, Error, RunHandle>({
    mutationFn: async (handle) => {
      const outcome = await handle.outcome;

      store.getState().runFinished(handle.runId, outcome);

      const stored = store.getState().result?.sessionId;

      update((written) => searchAfterRun(written, outcome, stored), { replace: true });

      return outcome;
    },
  });

  const run = useCallback(() => {
    const state = store.getState();

    if (selectRunning(state)) {
      return;
    }

    const request = selectRequest(state);

    const handle = client.startRun(request, (progress) => {
      store.getState().runProgressed(handle.runId, progress);
    });

    store.getState().runStarted(handle.runId, request, DateTime.now().toMillis());
    mutate(handle);
  }, [client, store, mutate]);

  const cancel = useCallback(() => {
    client.cancel();
  }, [client]);

  return { run, cancel };
}

export interface RunControl {
  run: () => void;
  cancel: () => void;
}
