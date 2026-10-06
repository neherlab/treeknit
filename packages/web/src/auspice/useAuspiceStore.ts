import type { AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";
import { createStateFromQueryOrJSONs } from "auspice/src/actions/recomputeReduxState";
import { CLEAN_START } from "auspice/src/actions/types";
import { useEffect, useState } from "react";

import type { Selection } from "../drawing/selection";
import { auspiceQuery } from "./query";
import { selectionMiddleware, treeSides } from "./selection";
import { type AuspiceMiddleware, type AuspiceStore, createAuspiceStore } from "./store";

export const START_QUERY = { legend: "closed" } as const satisfies Readonly<Record<string, string>>;

export interface AuspiceInput {
  datasets: AuspicePair;
  labels: readonly [string, string];
  trees: AuspiceTrees;
  query: Readonly<Record<string, string>>;
  onSelect: (selection: Selection) => void;
  onQuery: (query: string) => void;
}

export function useAuspiceStore({ onSelect, onQuery, ...input }: AuspiceInput): AuspiceStore {
  const [{ store, selected, queried }] = useState(() => {
    const selectListeners = new Set<(selection: Selection) => void>();
    const queryListeners = new Set<(query: string) => void>();

    const loaded = loadAuspiceStore({
      ...input,
      onSelect: (selection) => {
        for (const listener of selectListeners) {
          listener(selection);
        }
      },
      onQuery: (query) => {
        for (const listener of queryListeners) {
          listener(query);
        }
      },
    });

    return { store: loaded, selected: selectListeners, queried: queryListeners };
  });

  useEffect(() => {
    selected.add(onSelect);

    return () => {
      selected.delete(onSelect);
    };
  }, [selected, onSelect]);

  useEffect(() => {
    queried.add(onQuery);

    return () => {
      queried.delete(onQuery);
    };
  }, [queried, onQuery]);

  return store;
}

export function loadAuspiceStore({
  datasets,
  labels: [leftLabel, rightLabel],
  trees,
  query,
  onSelect,
  onQuery,
}: AuspiceInput): AuspiceStore {
  const store = createAuspiceStore(selectionMiddleware(treeSides(trees), onSelect), queryMiddleware(onQuery));

  const start = { ...START_QUERY, ...query };

  const state =
    trees === "both"
      ? createStateFromQueryOrJSONs({
          json: structuredClone(datasets.left),
          secondTreeDataset: structuredClone(datasets.right),
          mainTreeName: leftLabel,
          secondTreeName: rightLabel,
          query: start,
          dispatch: store.dispatch,
        })
      : createStateFromQueryOrJSONs({
          json: structuredClone(trees === "left" ? datasets.left : datasets.right),
          mainTreeName: trees === "left" ? leftLabel : rightLabel,
          query: start,
          dispatch: store.dispatch,
        });

  store.dispatch({ ...state, type: CLEAN_START });

  return store;
}

function queryMiddleware(onQuery: (query: string) => void): AuspiceMiddleware {
  let written: string | undefined;

  return (store) => (next) => (action) => {
    const result: unknown = next(action);
    const query = auspiceQuery(store.getState().controls);

    if (query !== written) {
      written = query;
      onQuery(query);
    }

    return result;
  };
}
