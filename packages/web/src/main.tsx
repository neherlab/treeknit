import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/600.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/600.css";
import "./index.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { AnalysisClient, startWorker } from "./analysis/client";
import { AnalysisClientContext } from "./analysis/context";
import { App } from "./App";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
});

const analysisClient = new AnalysisClient(startWorker);

const root = document.getElementById("root");

if (root !== null) {
  createRoot(root).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <AnalysisClientContext value={analysisClient}>
          <App />
        </AnalysisClientContext>
      </QueryClientProvider>
    </StrictMode>,
  );
}
