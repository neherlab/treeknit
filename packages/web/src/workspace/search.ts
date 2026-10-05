import type { LabelMode, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { stringifySearchWith } from "@tanstack/react-router";
import { isDeepEqual, omit } from "remeda";
import * as z from "zod";

import type { RunOutcome } from "../analysis/client";
import { everyVariantOf } from "../variants";

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

export const TREE_VERSIONS = everyVariantOf<TreeVersion>()(["input", "resolved", "imputed"]);

export const X_SCALES = everyVariantOf<Scale>()(["div", "depth"]);

export const LABEL_MODES = everyVariantOf<LabelMode>()(["auto", "on", "off"]);

export const NODE_SIDES = ["left", "right", "arg"] as const;

export type NodeSide = (typeof NODE_SIDES)[number];

const nodeRefSchema = z.object({ side: z.enum(NODE_SIDES), name: z.string().min(1) });

export type NodeRef = z.output<typeof nodeRefSchema>;

const nodeRefTextSchema = z
  .string()
  .transform((text) => {
    const [side, ...name] = text.split(":");

    return { side, name: name.join(":") };
  })
  .pipe(nodeRefSchema);

export const WORKSPACE_SEARCH_DEFAULTS = {
  view: "overview",
  pair: 0,
  version: "resolved",
  x: "div",
  labels: "auto",
} as const;

const integerSchema = z.union([
  z.int(),
  z
    .string()
    .regex(/^-?\d{1,15}$/u)
    .transform(Number),
]);

const nonNegativeIntegerSchema = integerSchema.pipe(z.int().min(0));

export const workspaceSearchSchema = z.object({
  view: choice(WORKSPACE_VIEWS, WORKSPACE_SEARCH_DEFAULTS.view),
  pair: nonNegativeInteger(WORKSPACE_SEARCH_DEFAULTS.pair),
  version: choice(TREE_VERSIONS, WORKSPACE_SEARCH_DEFAULTS.version),
  x: choice(X_SCALES, WORKSPACE_SEARCH_DEFAULTS.x),
  labels: choice(LABEL_MODES, WORKSPACE_SEARCH_DEFAULTS.labels),
  mcc: optional(nonNegativeIntegerSchema),
  leaf: optional(z.string().min(1)),
  node: optional(z.union([nodeRefSchema, nodeRefTextSchema])),
});

export type WorkspaceSearch = z.output<typeof workspaceSearchSchema>;

export function parseSearch(query: string): Record<string, string> {
  return Object.fromEntries(new URLSearchParams(query));
}

export const stringifySearch = stringifySearchWith(formatNodeRef);

export function formatNodeRef(node: NodeRef): string {
  return `${node.side}:${node.name}`;
}

export interface WorkspaceAvailability {
  hasResult: boolean;
  treeCount: number;
  resultTreeCount: number;
  pairCount: number;
  mccExists: (pair: number, mcc: number) => boolean;
  leafExists: (pair: number, leaf: string) => boolean;
  nodeExists: (pair: number, node: NodeRef) => boolean;
}

export const NO_WORKSPACE: WorkspaceAvailability = {
  hasResult: false,
  treeCount: 0,
  resultTreeCount: 0,
  pairCount: 0,
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
  tanglegram: ({ hasResult, pairCount }) => hasResult && pairCount > 0,
  auspice: ({ hasResult, pairCount }) => hasResult && pairCount > 0,
  arg: ({ hasResult }) => hasResult,
  mccs: ({ hasResult, pairCount }) => hasResult && pairCount > 0,
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

export function resolveWorkspaceSearch(search: WorkspaceSearch, availability: WorkspaceAvailability): WorkspaceSearch {
  const view = isViewAvailable(search.view, availability) ? search.view : WORKSPACE_SEARCH_DEFAULTS.view;
  const pair = resolvePair(search.pair, availability.pairCount);
  const keepMcc = search.mcc !== undefined && availability.mccExists(pair, search.mcc);
  const keepLeaf = search.leaf !== undefined && availability.leafExists(pair, search.leaf);
  const keepNode = search.node !== undefined && availability.nodeExists(pair, search.node);

  const resolved: WorkspaceSearch = { ...omit(search, ["mcc", "leaf", "node"]), view, pair };

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

export function resolvePair(pair: number, pairCount: number): number {
  return pair < pairCount ? pair : WORKSPACE_SEARCH_DEFAULTS.pair;
}

export function withSelectionPair(
  written: WorkspaceSearch,
  next: WorkspaceSearch,
  resolvedPair: number,
): WorkspaceSearch {
  const selectionChanged =
    next.mcc !== written.mcc || next.leaf !== written.leaf || !isDeepEqual(next.node, written.node);

  return selectionChanged && next.pair === written.pair ? { ...next, pair: resolvedPair } : next;
}

export function selectPair(search: WorkspaceSearch, pair: number): WorkspaceSearch {
  return { ...omit(search, ["mcc", "node"]), pair };
}

function choice<const V extends string>(values: readonly [V, ...V[]], fallback: NoInfer<V>) {
  return z.enum(values).default(fallback).catch(fallback);
}

function nonNegativeInteger(fallback: number) {
  return nonNegativeIntegerSchema.default(fallback).catch(fallback);
}

function optional<T extends z.ZodType>(schema: T) {
  return schema.optional().catch(undefined);
}

export function searchAfterRun(
  search: WorkspaceSearch,
  outcome: RunOutcome,
  storedSessionId: number | undefined,
): WorkspaceSearch {
  const stored = outcome.status === "succeeded" && outcome.sessionId === storedSessionId;

  return stored ? { ...search, view: "auspice" } : search;
}
