import { useMutation } from "@tanstack/react-query";
import { useCallback } from "react";

import { useAnalysisClient } from "../analysis/context";
import { useWorkspaceStore } from "./context";
import { type FinishedRun, runAnalysis } from "./runAnalysis";
import { searchAfterRun } from "./search";
import { useWorkspaceSearch } from "./useWorkspaceSearch";

export function useRunAnalysis(): RunControl {
  const client = useAnalysisClient();
  const store = useWorkspaceStore();
  const { update } = useWorkspaceSearch();

  const { mutate } = useMutation<FinishedRun | null, Error, Promise<FinishedRun | null>>({
    mutationFn: async (running) => {
      const finished = await running;

      if (finished !== null) {
        update((written) => searchAfterRun(written, finished.outcome, finished.storedSessionId, false), {
          replace: true,
        });
      }

      return finished;
    },
  });

  const run = useCallback(() => {
    mutate(runAnalysis(client, store));
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
