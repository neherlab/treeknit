import type { LabelMode, PairView, Scale, TreeVersion } from "@neherlab/treeknit-wasm";
import { lazy, useCallback, useEffect, useMemo, useState } from "react";
import { getErrorMessage } from "react-error-boundary";

import { usePairView } from "../analysis/queries";
import { CanvasBoundary } from "../canvas/CanvasBoundary";
import type { TreeView, TreeViewActions } from "../canvas/useTreeView";
import type { RowRange } from "../canvas/viewState";
import { ZoomControls } from "../canvas/ZoomControls";
import { FigureButton, LabelModeSelect, ScaleToggle, VersionToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { focusDone, type FocusTarget, focusRows, useFocusRequest } from "../drawing/focus";
import { LeafSearch } from "../drawing/LeafSearch";
import { selectionOf, type Selection, withSelection } from "../drawing/selection";
import { leafNames, leafRow, rowCount } from "../drawing/trees";
import { useDrawingView } from "../drawing/useDrawingView";
import { Select, type SelectOption } from "../ui/Select";
import { Switch } from "../ui/Switch";
import { useWorkspace } from "../workspace/context";
import { selectPair, type WorkspaceSearch } from "../workspace/search";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";

const TanglegramCanvas = lazy(async () => import("./TanglegramCanvas"));

export function TanglegramPanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Tanglegram result={result} />;
}

function Tanglegram({ result }: { result: RunResult }) {
  const { search, update } = useWorkspaceSearch();
  const { pair, version, x, labels } = search;
  const query = usePairView(result.sessionId, pair, version, x);
  const data = query.data;
  const rows = data === undefined ? 1 : rowCount(data.left, data.right);
  const view = useDrawingView(rows);
  const [colorByMcc, setColorByMcc] = useState(true);
  const selection = useMemo(() => selectionOf(search), [search]);
  const pairs = result.summary.pairs;
  const labelsOf = pairs[pair]?.labels;
  const resultKey = `${String(result.sessionId)}:tanglegram`;

  useFocusedRows(data, view);

  const names = useMemo(
    () => (data === undefined ? [] : [...new Set([...leafNames(data.left), ...leafNames(data.right)])]),
    [data],
  );

  const set = useCallback(
    (change: (written: WorkspaceSearch) => WorkspaceSearch) => {
      update(change);
    },
    [update],
  );

  const select = useCallback(
    (next: Selection) => {
      set((written) => withSelection(written, next));
    },
    [set],
  );

  const findLeaf = useCallback(
    (name: string) => {
      select({ leaf: name });

      const row = data === undefined ? undefined : leafRow(data.left, data.right, name);

      if (row !== undefined) {
        view.actions.panTo(row);
      }
    },
    [data, select, view.actions],
  );

  const pairOptions = useMemo<SelectOption<string>[]>(
    () => pairs.map(({ index, labels: [a, b] }) => ({ id: String(index), label: `${a} and ${b}` })),
    [pairs],
  );

  const choosePair = useCallback(
    (id: string) => {
      set((written) => selectPair(written, Number(id)));
    },
    [set],
  );

  const chooseVersion = useCallback(
    (next: TreeVersion) => {
      set((written) => ({ ...written, version: next }));
    },
    [set],
  );

  const chooseScale = useCallback(
    (next: Scale) => {
      set((written) => ({ ...written, x: next }));
    },
    [set],
  );

  const chooseLabels = useCallback(
    (next: LabelMode) => {
      set((written) => ({ ...written, labels: next }));
    },
    [set],
  );

  const clear = useCallback(() => {
    select({});
  }, [select]);

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
      <FigureButton />
    </>
  );

  return (
    <DrawingPanel
      toolbar={toolbar}
      loading={query.isPending ? "Loading the tanglegram" : undefined}
      error={query.isError ? (getErrorMessage(query.error) ?? String(query.error)) : undefined}
      errorTitle="The tanglegram could not be loaded"
      onEscape={clear}
    >
      {data === undefined ? null : (
        <CanvasBoundary resultKey={resultKey}>
          <TanglegramCanvas
            data={data}
            view={view}
            labels={labels}
            colorByMcc={colorByMcc}
            selection={selection}
            onSelect={select}
            resultKey={resultKey}
            label={`Tanglegram of ${labelsOf?.[0] ?? data.left.label} and ${labelsOf?.[1] ?? data.right.label} with ${String(data.mccs.length)} MCCs`}
          />
        </CanvasBoundary>
      )}
    </DrawingPanel>
  );
}

function useFocusedRows(data: PairView | undefined, view: TreeView) {
  const request = useFocusRequest();
  const [handled, setHandled] = useState<number | null>(null);

  if (request !== null && request.id !== handled && data !== undefined && view.frame !== undefined) {
    setHandled(request.id);
    applyFocus(view.actions, request.target, focusRows(data, request.target));
  }

  useEffect(() => {
    // oxlint-disable-next-line react-you-might-not-need-an-effect/no-event-handler -- the render above applied the request; clearing it in the shared store during render would update other subscribers mid-render, and leaving it would replay it on the next mount
    if (handled !== null) {
      focusDone(handled);
    }
  }, [handled]);
}

function applyFocus(actions: TreeViewActions, target: FocusTarget, range: RowRange | null) {
  if (range === null) {
    return;
  }

  if (target.kind === "leaf") {
    actions.panTo(range.first);
  } else {
    actions.fitRows(range);
  }
}
