import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/lato/400.css";
import "@fontsource/lato/400-italic.css";
import "@fontsource/lato/700.css";
import "@fontsource/noto-sans/400.css";
import "@fontsource/noto-sans/700.css";
import "./index.css";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { createAnalysisClient } from "./analysis/browserHost";
import { AnalysisClientContext } from "./analysis/context";
import { createQueryClient } from "./analysis/queryClient";
import { basePath, hashRouteHref } from "./pages";
import { PaletteStyles } from "./palette/PaletteStyles";
import { createAppRouter } from "./router";
import { CLIENT_THEME_PROVIDER_PROPS } from "./shell/theme";
import { startWorkspace } from "./workspace/runtime";
import { WorkspaceProvider } from "./workspace/WorkspaceProvider";

const root = document.getElementById("root");

if (root !== null) {
  const base = basePath(import.meta.url);
  const routeHref = hashRouteHref(window.location, base);

  if (routeHref !== null) {
    window.history.replaceState(window.history.state, "", routeHref);
  }

  const router = createAppRouter(base);
  const client = createAnalysisClient();
  const queryClient = createQueryClient();
  const workspace = startWorkspace(client, queryClient);

  createRoot(root).render(
    <StrictMode>
      <ThemeProvider {...CLIENT_THEME_PROVIDER_PROPS}>
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
