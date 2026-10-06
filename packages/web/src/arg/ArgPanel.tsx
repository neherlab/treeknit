import { useMemo } from "react";

import { useArgView } from "../analysis/queries";
import { CanvasBoundary } from "../canvas/CanvasBoundary";
import { lazyCanvas, useLazyCanvas } from "../canvas/lazyCanvas";
import { ZoomControls } from "../canvas/ZoomControls";
import { FigureButton, LabelModeSelect, ScaleToggle } from "../drawing/DrawingControls";
import { type DrawingNotice, DrawingPanel } from "../drawing/DrawingPanel";
import { figureOptions } from "../drawing/figure";
import { LeafSearch } from "../drawing/LeafSearch";
import { shownScaleNotice } from "../drawing/scale";
import { segmentLabels } from "../drawing/tooltip";
import { argLeafNames, argLeafRows, rowCount } from "../drawing/trees";
import { useDrawingSearch, useFindLeaf } from "../drawing/useDrawingSearch";
import { useDrawingView } from "../drawing/useDrawingView";
import { useFigureDownload } from "../drawing/useFigureDownload";
import { useWorkspace } from "../workspace/context";
import type { RunResult } from "../workspace/store";
import { argFailure } from "./outcome";

const ARG_CANVAS = lazyCanvas(async () => import("./ArgCanvas"));

const ARG_FIGURE = { kind: "arg" } as const;

export function ArgPanel() {
  const result = useWorkspace((state) => state.result);

  return result === null ? null : <Arg result={result} />;
}

function Arg({ result }: { result: RunResult }) {
  const [ArgCanvas, reloadArgCanvas] = useLazyCanvas(ARG_CANVAS);
  const { search, selection, select, clearOnEscape, chooseScale, chooseLabels } = useDrawingSearch();
  const { scale, labels } = search;
  const query = useArgView(result.sessionId, scale);
  const data = query.data ?? undefined;
  const rows = useMemo(() => (data === undefined ? 1 : rowCount(data)), [data]);
  const view = useDrawingView(rows);
  const failure = argFailure(result.summary.arg, query.data);
  const segments = useMemo(() => segmentLabels(result.request.trees), [result.request.trees]);
  const names = useMemo(() => (data === undefined ? [] : argLeafNames(data)), [data]);
  const findLeaf = useFindLeaf(data, argLeafRows, select, view.actions);

  const figure = useFigureDownload(result.sessionId, ARG_FIGURE, async (client, sessionId) =>
    client.argFigure(sessionId, figureOptions(search)),
  );

  const toolbar = (
    <>
      <ScaleToggle value={scale} onChange={chooseScale} />
      <LabelModeSelect value={labels} onChange={chooseLabels} />
      <LeafSearch names={names} onSelect={findLeaf} />
      <ZoomControls view={view} />
      <FigureButton {...figure.button} />
    </>
  );

  const shownScale = data?.scale;

  const notice = useMemo<DrawingNotice | undefined>(
    () =>
      failure === undefined
        ? shownScaleNotice(scale, shownScale, "arg")
        : {
            tone: "warning",
            text: `The ARG could not be built: ${failure}. The MCCs and the other files are still valid.`,
          },
    [failure, scale, shownScale],
  );

  return (
    <DrawingPanel
      toolbar={toolbar}
      failure={figure.failure}
      notice={notice}
      query={query}
      loading="Loading the ARG"
      errorTitle="The ARG could not be loaded"
      onEscape={clearOnEscape}
    >
      {(shown) =>
        shown === null ? null : (
          <CanvasBoundary onReset={reloadArgCanvas} resultKey={`${String(result.sessionId)}:arg`}>
            <ArgCanvas
              data={shown}
              view={view}
              labels={labels}
              selection={selection}
              onSelect={select}
              segments={segments}
              label={`Ancestral reassortment graph of ${segments[0]} and ${segments[1]}`}
            />
          </CanvasBoundary>
        )
      }
    </DrawingPanel>
  );
}
