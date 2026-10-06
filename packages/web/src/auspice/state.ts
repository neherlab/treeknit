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
  mainTreeNumTips?: number | undefined;
  title?: string | undefined;
}

export interface AuspiceTreeState {
  loaded: boolean;
  name?: string | undefined;
  nodes: AuspiceNode[] | null;
  idxOfInViewRootNode: number;
  idxOfFilteredRoot?: number | undefined;
  visibility: number[] | null;
  observedMutations?: ObservedMutations | undefined;
}

export interface AuspiceTreeTooState {
  loaded: boolean;
  name?: string | undefined;
  nodes?: AuspiceNode[] | null | undefined;
  idxOfInViewRootNode?: number | undefined;
  idxOfFilteredRoot?: number | undefined;
  visibility?: number[] | null | undefined;
  observedMutations?: ObservedMutations | undefined;
  tangleTipLookup?: readonly (readonly [number, number])[] | undefined;
}

export interface AuspiceNode {
  name: string;
  arrayIdx: number;
  hasChildren: boolean;
}

export interface AuspiceControlsState {
  defaults: AuspiceControlDefaults;
  colorBy: string;
  layout: string;
  tipLabelKey: string | symbol;
  selectedBranchLabel: string;
  showAllBranchLabels: boolean;
  distanceMeasure: string;
  focus: string | null;
  branchLengthsToDisplay: string;
  panelLayout: string;
  showTreeToo: string | false | undefined;
  showTangle: boolean;
  legendOpen?: boolean | undefined;
  panelsToDisplay: readonly string[];
  selectedNode: AuspiceSelectedNode | null;
  filters: Readonly<Record<string | symbol, readonly AuspiceFilterValue[]>>;
  performanceFlags: ReadonlyMap<string, boolean>;
}

export interface AuspiceFilterValue {
  value: string;
  active: boolean;
}

export interface AuspiceSelectedNode {
  name: string;
  idx: number;
  isBranch: boolean;
  treeId: AuspiceTreeId;
  existingFilterState?: "active" | "inactive" | null | undefined;
}

export interface AuspicePublication {
  author: string;
  title: string;
  journal: string;
  year: string;
  href: string;
}

export type ObservedMutations = Readonly<Record<string, number>>;

export interface AuspiceControlDefaults {
  colorBy: string;
  layout: string;
  tipLabelKey: string | symbol;
  selectedBranchLabel: string;
}
