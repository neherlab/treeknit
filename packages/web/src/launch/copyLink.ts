import type { LinkLimits } from "@neherlab/treeknit-wasm";

import { type SearchRecord, stringifySearch, VIEW_KEYS } from "../workspace/search";
import { writeQuery } from "../workspace/searchQuery";

export function inlineSessionLink(pageUrl: string, written: Readonly<SearchRecord>, inlineSession: string): string {
  const view = Object.fromEntries(
    Object.entries(written).filter(([key]) => VIEW_KEYS.some((viewKey) => viewKey === key)),
  );

  return `${pageUrl}${stringifySearch({ run: true, ...view })}#${writeQuery([{ key: "session", value: inlineSession }])}`;
}

export function copyCheck(link: string, limits: Pick<LinkLimits, "maxLinkChars" | "longLinkChars">): CopyCheck {
  const length = link.length;

  if (length > limits.maxLinkChars) {
    return { kind: "tooLong", length };
  }

  return { kind: "copy", length, long: length > limits.longLinkChars };
}

export type CopyCheck = { kind: "copy"; length: number; long: boolean } | { kind: "tooLong"; length: number };
