import type { AuspicePair } from "@neherlab/treeknit-wasm";
import { type ReactNode, useCallback, useMemo, useState } from "react";
import { omit } from "remeda";

import { useAuspiceFiles, useAuspiceView } from "../analysis/queries";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { LeafSearch } from "../drawing/LeafSearch";
import { shownScaleNotice } from "../drawing/scale";
import { useDrawingSearch } from "../drawing/useDrawingSearch";
import { PanelBoundary } from "../ui/PanelBoundary";
import { useWorkspace } from "../workspace/context";
import { selectPair } from "../workspace/search";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { leafNames, shownTreeLabels } from "./datasets";
import { parseAuspiceQuery, withAuspiceQuery } from "./query";
import type { StripChoice } from "./strip";
import { TreeStrip } from "./TreeStrip";

const AUSPICE_VIEW = lazyCanvas(async () => import("./AuspiceView"));

const AUSPICE_NOTES: readonly string[] = ["The tanglegram, the tables, and the files are still available."];

export function AuspicePanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Auspice result={result} />;
}

function Auspice({ result }: { result: RunResult }) {
  const [AuspiceView, reloadAuspiceView] = useLazyCanvas(AUSPICE_VIEW);
  const { search, selection, select, clearOnEscape, chooseVersion, chooseScale } = useDrawingSearch();
  const { update, pairLabels } = useWorkspaceSearch();
  const { pair, version, scale, show: trees } = search;
  const query = useAuspiceView(result.sessionId, pair, version, scale);
  const files = useAuspiceFiles(result.sessionId, pair, version, scale, trees).data;
  const pairs = result.summary.pairs;
  const labels = pairs[pair]?.labels;
  const treeLabels = useMemo(() => result.request.trees.map(({ label }) => label), [result.request.trees]);
  const shown = useMemo(() => ({ pair, trees }), [pair, trees]);
  const shownLabels = useMemo(() => shownTreeLabels(labels, trees), [labels, trees]);
  const urlQuery = useMemo(() => parseAuspiceQuery(search.auspice), [search.auspice]);
  const names = useMemo(() => (query.data === undefined ? [] : leafNames(query.data, trees)), [query.data, trees]);

  const chooseShown = useCallback(
    (next: StripChoice) => {
      update((written) => {
        const paired =
          next.pair === pair ? omit(written, ["show"]) : omit(selectPair(written, pairLabels, next.pair), ["show"]);

        return next.show === undefined ? paired : { ...paired, show: next.show };
      });
    },
    [update, pair, pairLabels],
  );

  const writeQuery = useCallback(
    (text: string) => {
      update((written) => withAuspiceQuery(written, text), { replace: true });
    },
    [update],
  );

  const findLeaf = useCallback(
    (name: string) => {
      select({ leaf: name });
    },
    [select],
  );

  const toolbar = (
    <>
      <TreeStrip labels={treeLabels} pairs={pairs} shown={shown} onChange={chooseShown} />
      <VersionToggle value={version} onChange={chooseVersion} />
      <ScaleToggle value={scale} onChange={chooseScale} />
      <LeafSearch names={names} onSelect={findLeaf} />
    </>
  );

  return (
    <DrawingPanel
      toolbar={toolbar}
      notice={shownScaleNotice(scale, query.data?.scale, "pair")}
      query={query}
      loading="Loading the Auspice view"
      errorTitle="The Auspice view could not be loaded"
      onEscape={clearOnEscape}
    >
      {(datasets) =>
        labels === undefined ? null : (
          <AuspiceBoundary
            resultKey={`${String(result.sessionId)}:${String(pair)}:${version}:${scale}`}
            onReset={reloadAuspiceView}
          >
            <KeyedAuspiceView datasets={datasets}>
              {(key) => (
                <AuspiceView
                  key={`${String(key)}:${trees}`}
                  datasets={datasets}
                  labels={labels}
                  trees={trees}
                  query={urlQuery}
                  selection={selection}
                  pair={pair}
                  onSelect={select}
                  onQuery={writeQuery}
                  files={files}
                  treeLabels={shownLabels}
                  axisTitle={datasets.axis_title}
                />
              )}
            </KeyedAuspiceView>
          </AuspiceBoundary>
        )
      }
    </DrawingPanel>
  );
}

function KeyedAuspiceView({ datasets, children }: { datasets: AuspicePair; children: (key: number) => ReactNode }) {
  return children(useDatasetsGeneration(datasets));
}

function useDatasetsGeneration(datasets: AuspicePair): number {
  const [shown, setShown] = useState({ datasets, generation: 0 });

  if (shown.datasets !== datasets) {
    const next = { datasets, generation: shown.generation + 1 };

    setShown(next);

    return next.generation;
  }

  return shown.generation;
}

function AuspiceBoundary({ resultKey, onReset, children }: AuspiceBoundaryProps) {
  const resetKeys = useMemo(() => [resultKey], [resultKey]);

  return (
    <PanelBoundary
      title="Auspice cannot draw these trees."
      notes={AUSPICE_NOTES}
      loading="Loading Auspice"
      resetKeys={resetKeys}
      onReset={onReset}
    >
      {children}
    </PanelBoundary>
  );
}

interface AuspiceBoundaryProps {
  resultKey: string;
  onReset: () => void;
  children: ReactNode;
}
