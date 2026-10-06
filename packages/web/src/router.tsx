import {
  createBrowserHistory,
  createRootRoute,
  createRoute,
  createRouter,
  stripSearchParams,
} from "@tanstack/react-router";

import { HelpPage } from "./help/HelpPage";
import { PAGES, routerBasepath } from "./pages";
import { AppShell } from "./shell/AppShell";
import { NotFound } from "./shell/NotFound";
import { Workspace } from "./shell/Workspace";
import { parseSearch, stringifySearch, WORKSPACE_SEARCH_DEFAULTS, workspaceSearchSchema } from "./workspace/search";

export function createAppRouter(base: string) {
  const rootRoute = createRootRoute({ component: AppShell, notFoundComponent: NotFound });

  const workspaceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: PAGES.workspace,
    validateSearch: workspaceSearchSchema,
    search: { middlewares: [stripSearchParams(WORKSPACE_SEARCH_DEFAULTS)] },
    component: Workspace,
  });

  const helpRoute = createRoute({ getParentRoute: () => rootRoute, path: PAGES.help, component: HelpPage });

  return createRouter({
    routeTree: rootRoute.addChildren([workspaceRoute, helpRoute]),
    history: createBrowserHistory(),
    basepath: routerBasepath(base),
    parseSearch,
    stringifySearch,
  });
}

export type AppRouter = ReturnType<typeof createAppRouter>;

declare module "@tanstack/react-router" {
  interface Register {
    router: AppRouter;
  }
}
