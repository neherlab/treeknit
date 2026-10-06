import { createMemoryHistory, createRootRoute, createRoute, createRouter, stripSearchParams } from "@tanstack/react-router";
import { describe, expect, test } from "vitest";

import { parseSearch, stringifySearch, WORKSPACE_SEARCH_DEFAULTS, workspaceSearchSchema } from "../search";

describe("the workspace route", () => {
  test("keeps the keys of a link through a change of the view", async () => {
    const router = workspaceRouter("/?example=h3n2-2017&gamma=3&run&view=overview");

    await router.navigate({ from: "/", to: "/", search: (previous) => ({ ...previous, view: "files" as const }) });

    expect(router.latestLocation.searchStr).toBe("?example=h3n2-2017&gamma=3&run&view=files");
  });

  test("leaves the default view keys out of the address", async () => {
    const router = workspaceRouter("/?view=mccs&scale=depth");

    await router.navigate({
      from: "/",
      to: "/",
      search: (previous) => ({ ...previous, view: "overview" as const, scale: "div" as const }),
    });

    expect(router.latestLocation.searchStr).toBe("");
  });
});

function workspaceRouter(url: string) {
  const rootRoute = createRootRoute();

  const workspaceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    validateSearch: workspaceSearchSchema,
    search: { middlewares: [stripSearchParams(WORKSPACE_SEARCH_DEFAULTS)] },
  });

  return createRouter({
    routeTree: rootRoute.addChildren([workspaceRoute]),
    history: createMemoryHistory({ initialEntries: [url] }),
    isServer: false,
    origin: "https://treeknit.example",
    parseSearch,
    stringifySearch,
  });
}
