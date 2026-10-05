import { createContext, useContext } from "react";

import type { AnalysisClient } from "./client";

export const AnalysisClientContext = createContext<AnalysisClient | null>(null);

export function useAnalysisClient(): AnalysisClient {
  const client = useContext(AnalysisClientContext);

  if (client === null) {
    throw new Error("useAnalysisClient needs an AnalysisClientContext provider.");
  }

  return client;
}
