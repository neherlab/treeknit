import type { IgnoredKey, LinkEntry } from "@neherlab/treeknit-wasm";

import { isAuspiceKey } from "../auspice/query";
import { type QueryEntry, readQuery } from "../workspace/searchQuery";

export function linkEntries(location: { search: string; hash: string }): QueryEntry[] {
  const query = readQuery(location.search.startsWith("?") ? location.search.slice(1) : location.search);
  const fragment = location.hash.startsWith("#") ? location.hash.slice(1) : location.hash;

  return fragment.includes("=") ? [...query, ...readQuery(fragment)] : query;
}

export function launchEntries(entries: readonly QueryEntry[]): LinkEntry[] {
  return entries.map(({ key, value }) => ({ key, value: value === true ? "" : value }));
}

export function namesView(entries: readonly QueryEntry[]): boolean {
  return entries.some(({ key }) => key === "view");
}

export function ignoredKeyNote({ key, suggestion, afterLocationQuery }: IgnoredKey): string | null {
  if (afterLocationQuery) {
    return `The key "${key}" seems to belong to the address before it: write & inside an address as %26.`;
  }

  if (isAuspiceKey(key)) {
    return `"${key}" is an Auspice setting: put Auspice settings inside the auspice key, for example auspice=c=mcc.`;
  }

  return suggestion === null ? null : `TreeKnit ignored the key "${key}"; did you mean "${suggestion}"?`;
}
