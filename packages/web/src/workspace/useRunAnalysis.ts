import type { AnalysisRequest } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { DateTime } from "luxon";
import { useCallback } from "react";

import type { RunOutcome } from "../analysis/client";
import { useAnalysisClient } from "../analysis/context";
import { useWorkspaceStore } from "./context";
import { selectRequest, selectRunning } from "./store";

export function useRunAnalysis(): RunControl {
  const client = useAnalysisClient();
  const store = useWorkspaceStore();

  const { mutate } = useMutation<RunOutcome, Error, AnalysisRequest>({
    mutationFn: async (request) => {
      const handle = client.startRun(request, (progress) => {
        store.getState().runProgressed(handle.runId, progress);
      });

      store.getState().runStarted(handle.runId, request, DateTime.now().toMillis());

      const outcome = await handle.outcome;

      store.getState().runFinished(handle.runId, outcome);

      return outcome;
    },
  });

  const run = useCallback(() => {
    const state = store.getState();

    if (!selectRunning(state)) {
      mutate(selectRequest(state));
    }
  }, [store, mutate]);

  const cancel = useCallback(() => {
    client.cancel();
  }, [client]);

  return { run, cancel };
}

export interface RunControl {
  run: () => void;
  cancel: () => void;
}
