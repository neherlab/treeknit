import type { AuspiceTrees } from "@neherlab/treeknit-wasm";
import { DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import type { AnyAction } from "redux";
import * as z from "zod";

import { NO_SELECTION, type Selection } from "../drawing/selection";
import type { TreeSide } from "../drawing/trees";
import type { AuspiceState, AuspiceTreeId } from "./state";
import type { AuspiceMiddleware } from "./store";

export const FROM_WORKSPACE = "fromWorkspace";

const SELECT_NODE_ACTION = z.object({ name: z.string(), idx: z.number(), treeId: z.enum(["LEFT", "RIGHT"]) });

export type TreeSides = Readonly<Record<AuspiceTreeId, TreeSide>>;

export const PAIR_SIDES: TreeSides = { LEFT: "left", RIGHT: "right" };

export function treeSides(trees: AuspiceTrees): TreeSides {
  return trees === "right" ? { LEFT: "right", RIGHT: "left" } : PAIR_SIDES;
}

export function auspiceSelection(state: AuspiceState, action: AnyAction, sides: TreeSides): Selection | undefined {
  if (action[FROM_WORKSPACE] === true) {
    return undefined;
  }

  if (action.type === DESELECT_NODE) {
    return NO_SELECTION;
  }

  const selected = action.type === SELECT_NODE ? SELECT_NODE_ACTION.safeParse(action) : undefined;

  if (selected?.success !== true) {
    return undefined;
  }

  const { name, idx, treeId } = selected.data;
  const nodes = treeId === "RIGHT" ? state.treeToo.nodes : state.tree.nodes;
  const node = nodes?.[idx];

  if (node?.name !== name) {
    return undefined;
  }

  return node.hasChildren ? { node: { side: sides[treeId], name: node.name } } : { leaf: node.name };
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
