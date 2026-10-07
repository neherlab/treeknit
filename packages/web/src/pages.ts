import { readQuery } from "./workspace/searchQuery";

export const PAGES = { workspace: "/", help: "/help" } as const;

export function basePath(moduleUrl: string): string {
  return new URL("../", moduleUrl).pathname;
}

export function hashRouteHref(location: PageLocation, base: string): string | null {
  if (!location.hash.startsWith("#/")) {
    return null;
  }

  const route = location.hash.slice(1);
  const queryStart = route.indexOf("?");
  const path = queryStart === -1 ? route : route.slice(0, queryStart);
  const hashQuery = queryStart === -1 ? "" : route.slice(queryStart + 1);
  const hashKeys = new Set(readQuery(hashQuery).map(({ key }) => key));
  const kept = queryParts(location.search).filter((part) => !hashKeys.has(readQuery(part)[0]?.key ?? ""));
  const query = [...kept, ...queryParts(hashQuery)].join("&");
  const search = query === "" ? "" : `?${query}`;

  return `${base}${path.slice(1)}${search}`;
}

export interface PageLocation {
  pathname: string;
  search: string;
  hash: string;
}

function queryParts(query: string): string[] {
  return (query.startsWith("?") ? query.slice(1) : query).split("&").filter((part) => part !== "");
}
