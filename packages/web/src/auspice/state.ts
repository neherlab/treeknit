import type { AnyAction } from "redux";
import type { ThunkAction } from "redux-thunk";

export interface AuspiceState {
  metadata: AuspiceMetadataState;
  tree: AuspiceTreeState;
  treeToo: AuspiceTreeTooState;
  controls: AuspiceControlsState;
}

export type AuspiceThunk = ThunkAction<void, AuspiceState, undefined, AnyAction>;

export type AuspiceTreeId = "LEFT" | "RIGHT";

export interface AuspiceMetadataState {
  loaded: boolean;
}

export interface AuspiceTreeState {
  loaded: boolean;
  name?: string | undefined;
  nodes: AuspiceNode[] | null;
  idxOfInViewRootNode: number;
  visibility: number[] | null;
}

export interface AuspiceTreeTooState {
  loaded: boolean;
  name?: string | undefined;
  nodes?: AuspiceNode[] | null | undefined;
  tangleTipLookup?: readonly (readonly [number, number])[] | undefined;
}

export interface AuspiceNode {
  name: string;
  arrayIdx: number;
  hasChildren: boolean;
}

export interface AuspiceControlsState {
  colorBy: string;
  layout: string;
  distanceMeasure: string;
  showTreeToo: string | false | undefined;
  showTangle: boolean;
  panelsToDisplay: readonly string[];
  selectedNode: AuspiceSelectedNode | null;
  filters: Readonly<Record<string, readonly AuspiceFilterValue[]>>;
  performanceFlags: ReadonlyMap<string, boolean>;
}

interface AuspiceFilterValue {
  value: string;
  active: boolean;
}

export interface AuspiceSelectedNode {
  name: string;
  idx: number;
  isBranch: boolean;
  treeId: AuspiceTreeId;
}

export interface AuspicePublication {
  author: string;
  title: string;
  journal: string;
  year: string;
  href: string;
}
