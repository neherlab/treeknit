import type { LabelMode, Scale } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import { getErrorMessage } from "react-error-boundary";

import { useArgView } from "../analysis/queries";
import { CanvasBoundary, useLazyCanvas } from "../canvas/CanvasBoundary";
import { ZoomControls } from "../canvas/ZoomControls";
import { FIGURE_PENDING, FigureButton, LabelModeSelect, ScaleToggle } from "../drawing/DrawingControls";
import { DrawingPanel } from "../drawing/DrawingPanel";
import { LeafSearch } from "../drawing/LeafSearch";
import { selectionOf, type Selection, withSelection } from "../drawing/selection";
import type { SegmentLabels } from "../drawing/tooltip";
import { rowCount } from "../drawing/trees";
import { useDrawingView } from "../drawing/useDrawingView";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { argFailure } from "./outcome";

const loadArgCanvas = async () => import("./ArgCanvas");

export function ArgPanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Arg result={result} />;
}

function Arg({ result }: { result: RunResult }) {
  const [ArgCanvas, reloadArgCanvas] = useLazyCanvas(loadArgCanvas);
  const { search, update } = useWorkspaceSearch();
  const { x, labels } = search;
  const query = useArgView(result.sessionId, x);
  const data = query.data ?? undefined;
  const view = useDrawingView(data === undefined ? 1 : rowCount(data));
  const selection = useMemo(() => selectionOf(search), [search]);
  const failure = argFailure(result.summary.arg, query.data);
  const [first, second] = result.request.trees;
  const segments = useMemo<SegmentLabels>(() => [first?.label ?? "A", second?.label ?? "B"], [first, second]);

  const names = useMemo(() => data?.nodes.flatMap((node) => (node.leaf ? [node.label] : [])) ?? [], [data]);

  const select = useCallback(
    (next: Selection) => {
      update((written) => withSelection(written, next));
    },
    [update],
  );

  const findLeaf = useCallback(
    (name: string) => {
      select({ leaf: name });

      const row = data?.nodes.find((node) => node.leaf && node.label === name)?.y;

      if (row !== undefined) {
        view.actions.panTo(row);
      }
    },
    [data, select, view.actions],
  );

  const chooseScale = useCallback(
    (next: Scale) => {
      update((written) => ({ ...written, x: next }));
    },
    [update],
  );

  const chooseLabels = useCallback(
    (next: LabelMode) => {
      update((written) => ({ ...written, labels: next }));
    },
    [update],
  );

  const clear = useCallback(() => {
    select({});
  }, [select]);

  const toolbar = (
    <>
      <ScaleToggle value={x} onChange={chooseScale} />
      <LabelModeSelect value={labels} onChange={chooseLabels} />
      <LeafSearch names={names} onSelect={findLeaf} />
      <ZoomControls view={view} />
      <FigureButton disabledReason={FIGURE_PENDING} />
    </>
  );

  const notice =
    failure === undefined
      ? undefined
      : `The ARG could not be built: ${failure}. The MCCs and the other files are still valid.`;

  return (
    <DrawingPanel
      toolbar={toolbar}
      notice={notice}
      loading={query.isPending ? "Loading the ARG" : undefined}
      error={query.isError ? (getErrorMessage(query.error) ?? String(query.error)) : undefined}
      errorTitle="The ARG could not be loaded"
      onEscape={clear}
    >
      {data === undefined ? null : (
        <CanvasBoundary onReset={reloadArgCanvas} resultKey={`${String(result.sessionId)}:arg`}>
          <ArgCanvas
            data={data}
            view={view}
            labels={labels}
            selection={selection}
            onSelect={select}
            segments={segments}
            label={`Ancestral reassortment graph of ${segments[0]} and ${segments[1]}`}
          />
        </CanvasBoundary>
      )}
    </DrawingPanel>
  );
}
