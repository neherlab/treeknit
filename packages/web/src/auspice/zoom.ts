export type ZoomAction = "out" | "selected" | "root";

export interface ZoomTree {
  inViewRoot: number;
  filteredRoot: number | undefined;
  parentOfInViewRoot: () => number;
}

export interface ZoomButtonStates {
  out: boolean;
  selected: boolean;
  root: boolean;
}

export type ZoomRoots = [number | undefined, number | undefined];

export function zoomButtonStates(trees: readonly ZoomTree[]): ZoomButtonStates {
  return {
    out: trees.some(isZoomed),
    selected: trees.some(isFiltered),
    root: trees.some(isZoomed),
  };
}

export function zoomRoots(action: ZoomAction, [main, second]: readonly [ZoomTree, ZoomTree | undefined]): ZoomRoots {
  return [zoomRoot(action, main), second === undefined ? undefined : zoomRoot(action, second)];
}

function zoomRoot(action: ZoomAction, tree: ZoomTree): number | undefined {
  if (action === "selected") {
    return isFiltered(tree) ? tree.filteredRoot : undefined;
  }

  if (!isZoomed(tree)) {
    return undefined;
  }

  return action === "out" ? tree.parentOfInViewRoot() : 0;
}

function isZoomed(tree: ZoomTree): boolean {
  return tree.inViewRoot !== 0;
}

function isFiltered(tree: ZoomTree): boolean {
  return tree.filteredRoot !== undefined && tree.filteredRoot !== 0 && tree.filteredRoot !== tree.inViewRoot;
}
