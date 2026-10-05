import { DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import type { AnyAction } from "redux";

import { NO_SELECTION, type Selection } from "../drawing/selection";
import type { AuspiceState, AuspiceTreeId } from "./state";
import type { AuspiceMiddleware } from "./store";

const TREE_SIDES = { LEFT: "left", RIGHT: "right" } as const satisfies Record<AuspiceTreeId, "left" | "right">;

export function auspiceSelection(state: AuspiceState, action: AnyAction): Selection | undefined {
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

  return node.hasChildren ? { node: { side: TREE_SIDES[action.treeId], name: node.name } } : { leaf: node.name };
}

export function selectionMiddleware(onSelect: (selection: Selection) => void): AuspiceMiddleware {
  return (store) => (next) => (action: AnyAction) => {
    const selection = auspiceSelection(store.getState(), action);
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
