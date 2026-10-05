import type { AuspicePair } from "@neherlab/treeknit-wasm";
import { createStateFromQueryOrJSONs } from "auspice/src/actions/recomputeReduxState";
import { CLEAN_START } from "auspice/src/actions/types";
import { useEffect, useState } from "react";

import type { Selection } from "../drawing/selection";
import { selectionMiddleware } from "./selection";
import { type AuspiceStore, createAuspiceStore } from "./store";

export const START_QUERY = { legend: "closed" } as const satisfies Readonly<Record<string, string>>;

export interface AuspiceInput {
  datasets: AuspicePair;
  labels: readonly [string, string];
  onSelect: (selection: Selection) => void;
}

export function useAuspiceStore({ datasets, labels, onSelect }: AuspiceInput): AuspiceStore {
  const [{ store, listeners }] = useState(() => {
    const selected = new Set<(selection: Selection) => void>();

    const loaded = loadAuspiceStore({
      datasets,
      labels,
      onSelect: (selection) => {
        for (const listener of selected) {
          listener(selection);
        }
      },
    });

    return { store: loaded, listeners: selected };
  });

  useEffect(() => {
    listeners.add(onSelect);

    return () => {
      listeners.delete(onSelect);
    };
  }, [listeners, onSelect]);

  return store;
}

export function loadAuspiceStore({ datasets, labels: [mainTreeName, secondTreeName], onSelect }: AuspiceInput) {
  const store = createAuspiceStore(selectionMiddleware(onSelect));

  const state = createStateFromQueryOrJSONs({
    json: structuredClone(datasets.left),
    secondTreeDataset: structuredClone(datasets.right),
    mainTreeName,
    secondTreeName,
    query: { ...START_QUERY },
    dispatch: store.dispatch,
  });

  store.dispatch({ ...state, type: CLEAN_START });

  return store;
}
