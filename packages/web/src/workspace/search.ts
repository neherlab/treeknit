import type { LabelMode, Scale, Version } from "@neherlab/treeknit-wasm";
import { stringifySearchWith } from "@tanstack/react-router";
import { omit } from "remeda";
import * as z from "zod";

export const WORKSPACE_VIEWS = [
  "overview",
  "tanglegram",
  "arg",
  "mccs",
  "constellation",
  "files",
  "diagnostics",
] as const;

export type WorkspaceView = (typeof WORKSPACE_VIEWS)[number];

export const TREE_VERSIONS = everyVariantOf<Version>()(["input", "resolved", "imputed"]);

export const X_SCALES = everyVariantOf<Scale>()(["div", "depth"]);

export const LABEL_MODES = everyVariantOf<LabelMode>()(["auto", "on", "off"]);

export const NODE_SIDES = ["left", "right", "arg"] as const;

export type NodeSide = (typeof NODE_SIDES)[number];

export interface NodeRef {
  side: NodeSide;
  name: string;
}

export const WORKSPACE_SEARCH_DEFAULTS = {
  view: "overview",
  pair: 0,
  version: "resolved",
  x: "div",
  labels: "auto",
} as const;

const NODE_REF_PATTERN = /^(left|right|arg):(.+)$/su;

const integerSchema = z.union([
  z.int(),
  z
    .string()
    .regex(/^-?\d{1,15}$/u)
    .transform(Number),
]);

export const workspaceSearchSchema = z.object({
  view: choice(WORKSPACE_VIEWS, WORKSPACE_SEARCH_DEFAULTS.view),
  pair: nonNegativeInteger(WORKSPACE_SEARCH_DEFAULTS.pair),
  version: choice(TREE_VERSIONS, WORKSPACE_SEARCH_DEFAULTS.version),
  x: choice(X_SCALES, WORKSPACE_SEARCH_DEFAULTS.x),
  labels: choice(LABEL_MODES, WORKSPACE_SEARCH_DEFAULTS.labels),
  mcc: optional(integerSchema),
  leaf: optional(z.string().min(1)),
  node: optional(z.string().regex(NODE_REF_PATTERN)),
});

export type WorkspaceSearch = z.output<typeof workspaceSearchSchema>;

export function parseSearch(query: string): Record<string, string> {
  return Object.fromEntries(new URLSearchParams(query));
}

export const stringifySearch = stringifySearchWith(String);

export function parseNodeRef(text: string): NodeRef | undefined {
  const match = NODE_REF_PATTERN.exec(text);
  const side = NODE_SIDES.find((candidate) => candidate === match?.[1]);
  const name = match?.[2];

  return side === undefined || name === undefined ? undefined : { side, name };
}

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

const VIEW_AVAILABLE: Record<WorkspaceView, (availability: WorkspaceAvailability) => boolean> = {
  overview: () => true,
  tanglegram: ({ hasResult, pairCount }) => hasResult && pairCount > 0,
  arg: ({ hasResult, resultTreeCount }) => hasResult && resultTreeCount === 2,
  mccs: ({ hasResult, pairCount }) => hasResult && pairCount > 0,
  constellation: ({ hasResult, resultTreeCount }) => hasResult && resultTreeCount >= 3,
  files: ({ hasResult }) => hasResult,
  diagnostics: ({ hasResult }) => hasResult,
};

export function isViewAvailable(view: WorkspaceView, availability: WorkspaceAvailability): boolean {
  return VIEW_AVAILABLE[view](availability);
}

export function resolveWorkspaceSearch(search: WorkspaceSearch, availability: WorkspaceAvailability): WorkspaceSearch {
  const view = isViewAvailable(search.view, availability) ? search.view : WORKSPACE_SEARCH_DEFAULTS.view;
  const pair = search.pair < availability.pairCount ? search.pair : WORKSPACE_SEARCH_DEFAULTS.pair;
  const node = search.node === undefined ? undefined : parseNodeRef(search.node);
  const keepMcc = search.mcc !== undefined && availability.mccExists(pair, search.mcc);
  const keepLeaf = search.leaf !== undefined && availability.leafExists(pair, search.leaf);
  const keepNode = node !== undefined && availability.nodeExists(pair, node);

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

export function selectPair(search: WorkspaceSearch, pair: number): WorkspaceSearch {
  return { ...omit(search, ["mcc", "node"]), pair };
}

function everyVariantOf<T extends string>() {
  return <const V extends readonly [T, ...T[]]>(values: V & ([Exclude<T, V[number]>] extends [never] ? unknown : never)) =>
    values;
}

function choice<const V extends string>(values: readonly [V, ...V[]], fallback: NoInfer<V>) {
  return z.enum(values).default(fallback).catch(fallback);
}

function nonNegativeInteger(fallback: number) {
  return integerSchema.pipe(z.int().min(0)).default(fallback).catch(fallback);
}

function optional<T extends z.ZodType>(schema: T) {
  return schema.optional().catch(undefined);
}
