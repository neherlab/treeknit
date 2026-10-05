import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-sans-condensed/400.css";
import "@fontsource/ibm-plex-sans-condensed/500.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/600.css";
import "./index.css";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { createAnalysisClient } from "./analysis/browserHost";
import { AnalysisClientContext } from "./analysis/context";
import { createQueryClient } from "./analysis/queryClient";
import { PaletteStyles } from "./palette/PaletteStyles";
import { router } from "./router";
import { startWorkspace } from "./workspace/runtime";
import { WorkspaceProvider } from "./workspace/WorkspaceProvider";

const root = document.getElementById("root");

if (root !== null) {
  const client = createAnalysisClient();
  const queryClient = createQueryClient();
  const workspace = startWorkspace(client, queryClient);

  createRoot(root).render(
    <StrictMode>
      <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
        <QueryClientProvider client={queryClient}>
          <AnalysisClientContext value={client}>
            <PaletteStyles />
            <WorkspaceProvider runtime={workspace}>
              <RouterProvider router={router} />
            </WorkspaceProvider>
          </AnalysisClientContext>
        </QueryClientProvider>
      </ThemeProvider>
    </StrictMode>,
  );
}
