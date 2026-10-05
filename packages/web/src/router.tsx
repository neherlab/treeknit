import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  stripSearchParams,
} from "@tanstack/react-router";

import { HelpPage } from "./help/HelpPage";
import { AppShell } from "./shell/AppShell";
import { NotFound } from "./shell/NotFound";
import { Workspace } from "./shell/Workspace";
import { parseSearch, stringifySearch, WORKSPACE_SEARCH_DEFAULTS, workspaceSearchSchema } from "./workspace/search";

const rootRoute = createRootRoute({ component: AppShell, notFoundComponent: NotFound });

const workspaceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch: workspaceSearchSchema,
  search: { middlewares: [stripSearchParams(WORKSPACE_SEARCH_DEFAULTS)] },
  component: Workspace,
});

const helpRoute = createRoute({ getParentRoute: () => rootRoute, path: "/help", component: HelpPage });

export const router = createRouter({
  routeTree: rootRoute.addChildren([workspaceRoute, helpRoute]),
  history: createHashHistory(),
  parseSearch,
  stringifySearch,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
