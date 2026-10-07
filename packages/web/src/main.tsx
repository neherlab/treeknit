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
import { loadExample } from "./analysis/example";
import { createQueryClient } from "./analysis/queryClient";
import { AddressBarSync, linkPairsClient, linkSearch } from "./launch/addressBar";
import { LaunchContext } from "./launch/context";
import { linkEntries } from "./launch/entries";
import { launchClient, LinkLaunch } from "./launch/LinkLaunch";
import { receiveSession } from "./launch/message";
import { windowPortal } from "./launch/windowPortal";
import { basePath, hashRouteHref, PAGES } from "./pages";
import { PaletteStyles } from "./palette/PaletteStyles";
import { createAppRouter } from "./router";
import { CLIENT_THEME_PROVIDER_PROPS } from "./shell/theme";
import { runAnalysis } from "./workspace/runAnalysis";
import { startWorkspace } from "./workspace/runtime";
import { searchAfterRun, workspaceSearchSchema } from "./workspace/search";
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
  const onWorkspacePage = router.latestLocation.pathname === PAGES.workspace;

  const launch = new LinkLaunch(
    {
      client: launchClient(client),
      fetchFile: fetch,
      loadExample,
      receiveSession: async (source) =>
        receiveSession(
          windowPortal(source),
          source === "opener" ? "page that opened TreeKnit" : "page that embeds TreeKnit",
        ),
      run: async (keepView) => {
        const finished = await runAnalysis(client, (await workspace).store);

        if (finished !== null) {
          await router.navigate({
            from: PAGES.workspace,
            to: PAGES.workspace,
            search: (previous) => ({
              ...previous,
              ...searchAfterRun(
                workspaceSearchSchema.parse(previous),
                finished.outcome,
                finished.storedSessionId,
                keepView,
              ),
            }),
            hash: true,
            replace: true,
          });
        }
      },
    },
    onWorkspacePage ? linkEntries(window.location) : [],
  );

  createRoot(root).render(
    <StrictMode>
      <ThemeProvider {...CLIENT_THEME_PROVIDER_PROPS}>
        <QueryClientProvider client={queryClient}>
          <AnalysisClientContext value={client}>
            <PaletteStyles />
            <WorkspaceProvider runtime={workspace}>
              <LaunchContext value={launch}>
                <RouterProvider router={router} />
              </LaunchContext>
            </WorkspaceProvider>
          </AnalysisClientContext>
        </QueryClientProvider>
      </ThemeProvider>
    </StrictMode>,
  );

  const runtime = await workspace.catch(() => null);

  if (runtime !== null) {
    const addressBar = new AddressBarSync(runtime.store, linkPairsClient(client), {
      onWorkspace: () => router.state.location.pathname === PAGES.workspace,
      write: async (pairs) =>
        router.navigate({
          from: PAGES.workspace,
          to: PAGES.workspace,
          search: (previous) => linkSearch(previous, pairs),
          replace: true,
        }),
    });

    router.subscribe("onResolved", ({ pathChanged }) => {
      if (pathChanged) {
        void addressBar.refresh(true);
      }
    });

    const launching = launch.start(runtime.store);

    await launch.settled;
    addressBar.start();
    await launching;
  }
}
