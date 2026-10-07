import type { LinkLimits } from "@neherlab/treeknit-wasm";
import { pick } from "remeda";

import { type SearchRecord, stringifySearch, VIEW_KEYS } from "../workspace/search";
import { writeQuery } from "../workspace/searchQuery";

export function inlineSessionLink(pageUrl: string, written: Readonly<SearchRecord>, inlineSession: string): string {
  return `${pageUrl}${stringifySearch({ run: true, ...pick(written, VIEW_KEYS) })}#${writeQuery([{ key: "session", value: inlineSession }])}`;
}

export function copyCheck(link: string, limits: Pick<LinkLimits, "maxLinkChars" | "longLinkChars">): CopyCheck {
  const length = link.length;

  if (length > limits.maxLinkChars) {
    return { kind: "tooLong", length };
  }

  return { kind: "copy", length, long: length > limits.longLinkChars };
}

export type CopyCheck = { kind: "copy"; length: number; long: boolean } | { kind: "tooLong"; length: number };
