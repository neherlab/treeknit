import type { Analysis, AnalysisRequest, Settings } from "@neherlab/treeknit-wasm";
import { useMutation, useSuspenseQuery } from "@tanstack/react-query";
import { createContext, useContext } from "react";

import type { AnalysisClient } from "./client";

export const AnalysisClientContext = createContext<AnalysisClient | undefined>(undefined);

export function useDefaultSettings(): Settings {
  const client = useAnalysisClient();

  return useSuspenseQuery({
    queryKey: ["defaultSettings"],
    queryFn: async () => client.defaultSettings(),
    staleTime: Number.POSITIVE_INFINITY,
  }).data;
}

export function useAnalysis() {
  const client = useAnalysisClient();

  return useMutation<Analysis, Error, AnalysisRequest>({
    mutationFn: async (request) => client.analyze(request),
  });
}

function useAnalysisClient(): AnalysisClient {
  const client = useContext(AnalysisClientContext);

  if (client === undefined) {
    throw new Error("useAnalysisClient needs an AnalysisClientContext provider");
  }

  return client;
}
