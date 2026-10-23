import type { AuspicePair, AuspiceTrees } from "@neherlab/treeknit-wasm";
import { createStateFromQueryOrJSONs } from "auspice/src/actions/recomputeReduxState";
import { CLEAN_START } from "auspice/src/actions/types";
import { useEffect, useState } from "react";

import type { Selection } from "../drawing/selection";
import { type AuspiceNotice, noticeMiddleware } from "./notices";
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
  onNotice: (notice: AuspiceNotice) => void;
}

export function useAuspiceStore({ onSelect, onQuery, onNotice, ...input }: AuspiceInput): AuspiceStore {
  const [{ store, selected, queried, noticed }] = useState(() => {
    const listeners = {
      selected: new Set<(selection: Selection) => void>(),
      queried: new Set<(query: string) => void>(),
      noticed: new Set<(notice: AuspiceNotice) => void>(),
    };

    const loaded = loadAuspiceStore({
      ...input,
      onSelect: (selection) => {
        notify(listeners.selected, selection);
      },
      onQuery: (query) => {
        notify(listeners.queried, query);
      },
      onNotice: (notice) => {
        notify(listeners.noticed, notice);
      },
    });

    return { store: loaded, ...listeners };
  });

  useListener(selected, onSelect);
  useListener(queried, onQuery);
  useListener(noticed, onNotice);

  return store;
}

export function loadAuspiceStore({
  datasets,
  labels: [leftLabel, rightLabel],
  trees,
  query,
  onSelect,
  onQuery,
  onNotice,
}: AuspiceInput): AuspiceStore {
  const store = createAuspiceStore(
    selectionMiddleware(treeSides(trees), onSelect),
    queryMiddleware(onQuery),
    noticeMiddleware(onNotice),
  );

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

function notify<T>(listeners: ReadonlySet<(value: T) => void>, value: T): void {
  for (const listener of listeners) {
    listener(value);
  }
}

function useListener<T>(listeners: Set<(value: T) => void>, listener: (value: T) => void): void {
  useEffect(() => {
    listeners.add(listener);

    return () => {
      listeners.delete(listener);
    };
  }, [listeners, listener]);
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
