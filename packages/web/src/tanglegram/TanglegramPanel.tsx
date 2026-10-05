import type { PairView, TreeVersion } from "@neherlab/treeknit-wasm";
import { useCallback, useEffect, useMemo, useState } from "react";

import { usePairView } from "../analysis/queries";
import { CanvasBoundary } from "../canvas/CanvasBoundary";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { type TreeViewActions, type TreeViewHandle, useTreeViewReady } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import { ZoomControls } from "../canvas/ZoomControls";
import { FigureButton, LabelModeSelect, ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { figureOptions } from "../drawing/figure";
import {
  focusApplies,
  focusDone,
  type FocusTarget,
  focusRows,
  revealLeafRows,
  useFocusRequest,
} from "../drawing/focus";
import { counted } from "../drawing/format";
import { LeafSearch } from "../drawing/LeafSearch";
import { pairLeafNames, pairLeafRows, rowCount } from "../drawing/trees";
import { useDrawingSearch, useFindLeaf } from "../drawing/useDrawingSearch";
import { useDrawingView } from "../drawing/useDrawingView";
import { useFigureDownload } from "../drawing/useFigureDownload";
import { Select, type SelectOption } from "../ui/Select";
import { Switch } from "../ui/Switch";
import { useWorkspace } from "../workspace/context";
import { selectPair } from "../workspace/search";
import type { RunResult } from "../workspace/store";

const TANGLEGRAM_CANVAS = lazyCanvas(async () => import("./TanglegramCanvas"));

export function TanglegramPanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Tanglegram result={result} />;
}

function Tanglegram({ result }: { result: RunResult }) {
  const [TanglegramCanvas, reloadTanglegramCanvas] = useLazyCanvas(TANGLEGRAM_CANVAS);
  const { search, update, selection, select, clear, chooseScale, chooseLabels } = useDrawingSearch();
  const { pair, version, x, labels } = search;
  const query = usePairView(result.sessionId, pair, version, x);
  const data = query.data;
  const rows = useMemo(() => (data === undefined ? 1 : rowCount(data.left, data.right)), [data]);
  const view = useDrawingView(rows);
  const [colorByMcc, setColorByMcc] = useState(true);
  const pairs = result.summary.pairs;
  const labelsOf = pairs[pair]?.labels;
  const resultKey = `${String(result.sessionId)}:tanglegram`;
  const names = useMemo(() => (data === undefined ? [] : pairLeafNames(data)), [data]);
  const findLeaf = useFindLeaf(data, leafRowsOf, select, view.actions);

  const figure = useFigureDownload(result.sessionId, { kind: "pair", pair }, async (client, sessionId) =>
    client.figure(sessionId, pair, version, figureOptions(search)),
  );

  useFocusedRows(data, pair, view);

  const pairOptions = useMemo<SelectOption<string>[]>(
    () => pairs.map(({ index, labels: [a, b] }) => ({ id: String(index), label: `${a} and ${b}` })),
    [pairs],
  );

  const choosePair = useCallback(
    (id: string) => {
      update((written) => selectPair(written, Number(id)));
    },
    [update],
  );

  const chooseVersion = useCallback(
    (next: TreeVersion) => {
      update((written) => ({ ...written, version: next }));
    },
    [update],
  );

  const toolbar = (
    <>
      {pairs.length > 1 ? (
        <Select
          label="Pair"
          labelHidden
          options={pairOptions}
          value={String(pair)}
          onChange={choosePair}
          className="w-48"
        />
      ) : null}
      <VersionToggle value={version} onChange={chooseVersion} />
      <ScaleToggle value={x} onChange={chooseScale} />
      <Switch label="Color branches by MCC" isSelected={colorByMcc} onChange={setColorByMcc} />
      <LabelModeSelect value={labels} onChange={chooseLabels} />
      <LeafSearch names={names} onSelect={findLeaf} />
      <ZoomControls view={view} />
      <FigureButton {...figure.button} />
    </>
  );

  return (
    <DrawingPanel
      toolbar={toolbar}
      failure={figure.failure}
      query={query}
      loading="Loading the tanglegram"
      errorTitle="The tanglegram could not be loaded"
      onEscape={clear}
    >
      {(shown) => (
        <CanvasBoundary onReset={reloadTanglegramCanvas} resultKey={resultKey}>
          <TanglegramCanvas
            data={shown}
            view={view}
            labels={labels}
            colorByMcc={colorByMcc}
            selection={selection}
            onSelect={select}
            resultKey={resultKey}
            label={`Tanglegram of ${labelsOf?.[0] ?? shown.left.label} and ${labelsOf?.[1] ?? shown.right.label} with ${counted(shown.mccs.length, "MCC", "MCCs")}`}
          />
        </CanvasBoundary>
      )}
    </DrawingPanel>
  );
}

function leafRowsOf(data: PairView, name: string): RowRange | null {
  return pairLeafRows(data.left, data.right, name);
}

function useFocusedRows(data: PairView | undefined, pair: number, view: TreeViewHandle) {
  const request = useFocusRequest();
  const ready = useTreeViewReady(view);
  const { actions } = view;

  useEffect(() => {
    // oxlint-disable-next-line react-you-might-not-need-an-effect/no-event-handler -- a focus request from another view waits in the focus store until the pair view has loaded and the canvas has a size; applying it writes the view store, which must not change during render
    if (request === null || data === undefined || !ready || !focusApplies(request, pair)) {
      return;
    }

    applyFocus(actions, request.target, focusRows(data, request.target));
    focusDone(request.id);
  }, [request, data, pair, ready, actions]);
}

function applyFocus(actions: TreeViewActions, target: FocusTarget, range: RowRange | null) {
  if (range === null) {
    return;
  }

  if (target.kind === "leaf") {
    revealLeafRows(actions, range);
  } else {
    actions.fitRows(range);
  }
}
