import { DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import type { AnyAction } from "redux";

import { NO_SELECTION, type Selection } from "../drawing/selection";
import type { TreeSide } from "../drawing/trees";
import type { AuspiceState, AuspiceTreeId } from "./state";
import type { AuspiceMiddleware } from "./store";

export const FROM_WORKSPACE = "fromWorkspace";

export type TreeSides = Readonly<Record<AuspiceTreeId, TreeSide>>;

export const PAIR_SIDES: TreeSides = { LEFT: "left", RIGHT: "right" };

export function treeSides(trees: "both" | "left" | "right"): TreeSides {
  return trees === "right" ? { LEFT: "right", RIGHT: "left" } : PAIR_SIDES;
}

export function auspiceSelection(state: AuspiceState, action: AnyAction, sides: TreeSides): Selection | undefined {
  if (action[FROM_WORKSPACE] === true) {
    return undefined;
  }

  if (action.type === DESELECT_NODE) {
    return NO_SELECTION;
  }

  if (action.type !== SELECT_NODE || !isSelectNode(action)) {
    return undefined;
  }

  const nodes = action.treeId === "RIGHT" ? state.treeToo.nodes : state.tree.nodes;
  const node = nodes?.[action.idx];

  if (node?.name !== action.name) {
    return undefined;
  }

  return node.hasChildren ? { node: { side: sides[action.treeId], name: node.name } } : { leaf: node.name };
}

export function selectionMiddleware(sides: TreeSides, onSelect: (selection: Selection) => void): AuspiceMiddleware {
  return (store) => (next) => (action: AnyAction) => {
    const selection = auspiceSelection(store.getState(), action, sides);
    const result: unknown = next(action);

    if (selection !== undefined) {
      onSelect(selection);
    }

    return result;
  };
}

interface SelectNodeAction {
  name: string;
  idx: number;
  treeId: AuspiceTreeId;
}

function isSelectNode(action: AnyAction): action is AnyAction & SelectNodeAction {
  return (
    typeof action["name"] === "string" &&
    typeof action["idx"] === "number" &&
    (action["treeId"] === "LEFT" || action["treeId"] === "RIGHT")
  );
}
