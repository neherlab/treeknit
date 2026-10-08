import { createMemoryHistory, createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { describe, expect, test } from "vitest";

import { basePath, hashRouteHref, PAGES } from "../pages";

describe("basePath", () => {
  test.each([
    [
      "the entry chunk of a build on GitHub Pages",
      "https://neherlab.github.io/treeknit/assets/index-1a2b.js",
      "/treeknit/",
    ],
    ["the entry chunk of a build at the root of a host", "https://example.org/assets/index-1a2b.js", "/"],
    ["the entry module of the dev server", "http://localhost:6180/src/main.tsx", "/"],
  ])("is the directory above %s", (_case, moduleUrl, expected) => {
    expect(basePath(moduleUrl)).toBe(expected);
  });
});

describe("the router with the base path", () => {
  test.each([
    ["/treeknit/", "/"],
    ["/treeknit/help", "/help"],
    ["/treeknit/?view=mccs", "/"],
  ])("lets the router match %s as %s", (url, route) => {
    expect(matchedRoute("/treeknit/", url)).toBe(route);
  });

  test("lets the router write the base path into links", () => {
    const router = testRouter("/treeknit/", "/treeknit/");

    expect(router.buildLocation({ to: PAGES.help }).publicHref).toBe("/treeknit/help");
  });
});

describe("hashRouteHref", () => {
  test.each([
    ["the workspace", { search: "", hash: "#/" }, "/treeknit/"],
    ["the help page", { search: "", hash: "#/help" }, "/treeknit/help"],
    ["view keys", { search: "", hash: "#/?view=mccs&labels=on" }, "/treeknit/?view=mccs&labels=on"],
    [
      "a real query next to the hash query, hash keys winning",
      { search: "?ref=paper&view=files", hash: "#/?view=mccs" },
      "/treeknit/?ref=paper&view=mccs",
    ],
    ["a real query alone", { search: "?fbclid=x", hash: "#/help" }, "/treeknit/help?fbclid=x"],
  ])("moves %s of an old hash link into the path", (_case, location, expected) => {
    expect(hashRouteHref({ pathname: "/treeknit/", ...location }, "/treeknit/")).toBe(expected);
  });

  test.each([
    ["a help anchor", "#help-cite"],
    ["no fragment", ""],
    ["an inline session", "#session=data:application/gzip;base64,H4sI"],
  ])("leaves %s alone", (_case, hash) => {
    expect(hashRouteHref({ pathname: "/treeknit/help", search: "", hash }, "/treeknit/")).toBeNull();
  });
});

function matchedRoute(base: string, url: string): string | undefined {
  const router = testRouter(base, url);

  return router.matchRoutes(router.latestLocation.pathname, router.latestLocation.search).at(-1)?.routeId;
}

function testRouter(base: string, url: string) {
  const rootRoute = createRootRoute();
  const pages = Object.values(PAGES).map((path) => createRoute({ getParentRoute: () => rootRoute, path }));

  return createRouter({
    routeTree: rootRoute.addChildren(pages),
    history: createMemoryHistory({ initialEntries: [url] }),
    basepath: base,
  });
}
