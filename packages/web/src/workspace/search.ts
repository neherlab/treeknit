import type { AuspiceTrees, LabelMode, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { LABEL_MODE_VALUES, SCALE_VALUES, TREE_VERSION_VALUES } from "@neherlab/treeknit-wasm/variants";
import { isDeepEqual, omit } from "remeda";
import * as z from "zod";

import type { RunOutcome } from "../analysis/client";
import { type QueryRecord, queryRecord, queryValueSchema, readQuery, recordEntries, writeQuery } from "./searchQuery";

export const WORKSPACE_VIEWS = [
  "overview",
  "tanglegram",
  "auspice",
  "arg",
  "mccs",
  "constellation",
  "files",
  "diagnostics",
] as const;

export type WorkspaceView = (typeof WORKSPACE_VIEWS)[number];

export const NODE_SIDES = ["left", "right", "arg"] as const;

export type NodeSide = (typeof NODE_SIDES)[number];

const nonEmptyText = z.string().min(1);

const nodeRefSchema = z.object({ side: z.enum(NODE_SIDES), name: nonEmptyText });

export type NodeRef = z.output<typeof nodeRefSchema>;

const pairRefSchema = z.tuple([nonEmptyText, nonEmptyText]).readonly();

export type PairRef = z.output<typeof pairRefSchema>;

export const WORKSPACE_SEARCH_DEFAULTS = {
  view: "overview",
  version: "resolved",
  scale: "div",
  labels: "auto",
} as const;

const pairCodec = z.codec(z.string(), pairRefSchema, {
  decode: (text, payload) => {
    const parsed = pairRefSchema.safeParse(text.split(":"));

    if (parsed.success) {
      return parsed.data;
    }

    payload.issues.push({ code: "custom", message: "A pair is two tree labels separated by a colon.", input: text });

    return z.NEVER;
  },
  encode: ([first, second]) => `${first}:${second}`,
});

const nodeCodec = z.codec(z.string(), nodeRefSchema, {
  decode: (text, payload) => {
    const [side, ...name] = text.split(":");
    const parsed = nodeRefSchema.safeParse({ side, name: name.join(":") });

    if (parsed.success) {
      return parsed.data;
    }

    payload.issues.push({
      code: "custom",
      message: "A node is a side and a node name separated by a colon.",
      input: text,
    });

    return z.NEVER;
  },
  encode: ({ side, name }) => `${side}:${name}`,
});

const mccCodec = z.codec(z.string().regex(/^\d{1,15}$/u), z.int().min(0), {
  decode: (text) => Number(text) - 1,
  encode: (index) => String(index + 1),
});

export const VIEW_KEYS = [
  "view",
  "pair",
  "version",
  "scale",
  "labels",
  "show",
  "auspice",
  "mcc",
  "leaf",
  "node",
] as const;

type ViewKey = (typeof VIEW_KEYS)[number];

const VIEW_KEY_CODECS: Readonly<Record<ViewKey, z.ZodType<SearchValue, string>>> = {
  view: z.enum(WORKSPACE_VIEWS),
  pair: pairCodec,
  version: z.enum(TREE_VERSION_VALUES),
  scale: z.enum(SCALE_VALUES),
  labels: z.enum(LABEL_MODE_VALUES),
  show: nonEmptyText,
  auspice: nonEmptyText,
  mcc: mccCodec,
  leaf: nonEmptyText,
  node: nodeCodec,
};

export const workspaceSearchSchema = z.object({
  view: choice(WORKSPACE_VIEWS, WORKSPACE_SEARCH_DEFAULTS.view),
  pair: optional(pairRefSchema),
  version: choice(TREE_VERSION_VALUES, WORKSPACE_SEARCH_DEFAULTS.version),
  scale: choice(SCALE_VALUES, WORKSPACE_SEARCH_DEFAULTS.scale),
  labels: choice(LABEL_MODE_VALUES, WORKSPACE_SEARCH_DEFAULTS.labels),
  show: optional(nonEmptyText),
  auspice: optional(nonEmptyText),
  mcc: optional(z.int().min(0)),
  leaf: optional(nonEmptyText),
  node: optional(nodeRefSchema),
});

export type WrittenSearch = z.output<typeof workspaceSearchSchema>;

export interface WorkspaceSearch {
  view: WorkspaceView;
  pair: number;
  version: TreeVersion;
  scale: Scale;
  labels: LabelMode;
  show: AuspiceTrees;
  auspice?: string | undefined;
  mcc?: number | undefined;
  leaf?: string | undefined;
  node?: NodeRef | undefined;
}

export function parseSearch(query: string): SearchRecord {
  return Object.fromEntries(
    Object.entries(queryRecord(readQuery(query.startsWith("?") ? query.slice(1) : query))).flatMap(
      ([key, value]): [string, SearchValue][] => {
        if (!isViewKey(key)) {
          return [[key, value]];
        }

        const text = z.string().safeParse(value);
        const decoded = text.success ? z.safeDecode(VIEW_KEY_CODECS[key], text.data) : undefined;

        return decoded?.success === true ? [[key, decoded.data]] : [];
      },
    ),
  );
}

export function stringifySearch(search: Readonly<SearchRecord>): string {
  const record: QueryRecord = {};

  for (const [key, value] of Object.entries(search)) {
    const written = isViewKey(key) ? encodeViewKey(key, value) : queryValueSchema.safeParse(value).data;

    if (written !== undefined) {
      record[key] = written;
    }
  }

  const query = writeQuery(recordEntries(record));

  return query === "" ? "" : `?${query}`;
}

export type SearchRecord = Record<string, SearchValue>;

export type SearchValue = QueryRecord[string] | number | PairRef | NodeRef | undefined;

export function formatNodeRef(node: NodeRef): string {
  return z.encode(nodeCodec, node);
}

export interface WorkspaceAvailability {
  hasResult: boolean;
  treeCount: number;
  resultTreeCount: number;
  pairLabels: readonly PairRef[];
  mccExists: (pair: number, mcc: number) => boolean;
  leafExists: (pair: number, leaf: string) => boolean;
  nodeExists: (pair: number, node: NodeRef) => boolean;
}

export const NO_WORKSPACE: WorkspaceAvailability = {
  hasResult: false,
  treeCount: 0,
  resultTreeCount: 0,
  pairLabels: [],
  mccExists: () => false,
  leafExists: () => false,
  nodeExists: () => false,
};

const VIEW_TREE_COUNT: Partial<Record<WorkspaceView, (treeCount: number) => boolean>> = {
  arg: (treeCount) => treeCount === 2,
  constellation: (treeCount) => treeCount >= 3,
};

const VIEW_AVAILABLE: Record<WorkspaceView, (availability: WorkspaceAvailability) => boolean> = {
  overview: () => true,
  tanglegram: ({ hasResult, pairLabels }) => hasResult && pairLabels.length > 0,
  auspice: ({ hasResult, pairLabels }) => hasResult && pairLabels.length > 0,
  arg: ({ hasResult }) => hasResult,
  mccs: ({ hasResult, pairLabels }) => hasResult && pairLabels.length > 0,
  constellation: ({ hasResult }) => hasResult,
  files: ({ hasResult }) => hasResult,
  diagnostics: ({ hasResult }) => hasResult,
};

export function viewFitsTreeCount(view: WorkspaceView, treeCount: number): boolean {
  return VIEW_TREE_COUNT[view]?.(treeCount) ?? true;
}

export function isViewAvailable(view: WorkspaceView, availability: WorkspaceAvailability): boolean {
  return viewFitsTreeCount(view, availability.resultTreeCount) && VIEW_AVAILABLE[view](availability);
}

export function resolveWorkspaceSearch(search: WrittenSearch, availability: WorkspaceAvailability): WorkspaceSearch {
  const view = isViewAvailable(search.view, availability) ? search.view : WORKSPACE_SEARCH_DEFAULTS.view;
  const pair = resolvePair(search.pair, availability.pairLabels);
  const keepMcc = search.mcc !== undefined && availability.mccExists(pair, search.mcc);
  const keepLeaf = search.leaf !== undefined && availability.leafExists(pair, search.leaf);
  const keepNode = search.node !== undefined && availability.nodeExists(pair, search.node);

  const resolved: WorkspaceSearch = {
    ...omit(search, ["pair", "show", "mcc", "leaf", "node"]),
    view,
    pair,
    show: resolveShown(search.show, availability.pairLabels[pair]),
  };

  if (keepMcc) {
    resolved.mcc = search.mcc;
  }

  if (keepLeaf) {
    resolved.leaf = search.leaf;
  }

  if (keepNode) {
    resolved.node = search.node;
  }

  return resolved;
}

export function resolvePair(pair: PairRef | undefined, pairLabels: readonly PairRef[]): number {
  if (pair === undefined) {
    return 0;
  }

  const [a, b] = pair;

  const index = pairLabels.findIndex(
    ([first, second]) => (a === first && b === second) || (a === second && b === first),
  );

  return Math.max(index, 0);
}

export function resolveShown(show: string | undefined, labels: PairRef | undefined): AuspiceTrees {
  if (show === undefined || labels === undefined) {
    return "both";
  }

  if (show === labels[0]) {
    return "left";
  }

  return show === labels[1] ? "right" : "both";
}

export function shownLabel(trees: AuspiceTrees, labels: PairRef | undefined): string | undefined {
  if (trees === "both" || labels === undefined) {
    return undefined;
  }

  return trees === "left" ? labels[0] : labels[1];
}

export function withSelectionPair(
  written: WrittenSearch,
  next: WrittenSearch,
  pairLabels: readonly PairRef[],
  resolvedPair: number,
): WrittenSearch {
  const selectionChanged =
    next.mcc !== written.mcc || next.leaf !== written.leaf || !isDeepEqual(next.node, written.node);

  return selectionChanged && isDeepEqual(next.pair, written.pair)
    ? withPair(next, pairLabels[resolvedPair], resolvedPair)
    : next;
}

export function selectPair(search: WrittenSearch, pairLabels: readonly PairRef[], pair: number): WrittenSearch {
  return withPair(omit(search, ["mcc", "node"]), pairLabels[pair], pair);
}

function withPair(search: WrittenSearch, labels: PairRef | undefined, index: number): WrittenSearch {
  const rest = omit(search, ["pair"]);

  return index === 0 || labels === undefined ? rest : { ...rest, pair: labels };
}

function choice<const V extends string>(values: readonly [V, ...V[]], fallback: NoInfer<V>) {
  return z.enum(values).default(fallback).catch(fallback);
}

function optional<T extends z.ZodType>(schema: T) {
  return schema.optional().catch(undefined);
}

function isViewKey(key: string): key is ViewKey {
  return VIEW_KEYS.some((viewKey) => viewKey === key);
}

function encodeViewKey(key: ViewKey, value: SearchValue): string | undefined {
  if (value === undefined) {
    return undefined;
  }

  const encoded = z.safeEncode(VIEW_KEY_CODECS[key], value);

  return encoded.success ? encoded.data : undefined;
}

export function searchAfterRun(
  search: WrittenSearch,
  outcome: RunOutcome,
  storedSessionId: number | undefined,
  keepView: boolean,
): WrittenSearch {
  const stored = outcome.status === "succeeded" && outcome.sessionId === storedSessionId;

  return stored && !keepView ? { ...search, view: "auspice" } : search;
}
