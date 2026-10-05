import { type LayersList, OrthographicView } from "@deck.gl/core";
import { PolygonLayer } from "@deck.gl/layers";
import { DeckGL } from "@deck.gl/react";
import { cva } from "class-variance-authority";
import { useMemo } from "react";
import { mergeProps, useMove, usePress } from "react-aria";

import { withOpacity } from "./color";
import { useDrawingColors } from "./drawingColors";
import type { TreeView } from "./useTreeView";
import { type CanvasFrame, leafTarget, minimapLeafAt, minimapViewState, visibleWorldRect } from "./viewState";

const MINIMAP_VIEW = new OrthographicView({ id: "minimap", flipY: true });

const WINDOW_FILL_OPACITY = 0.12;

const minimapStyle = cva(
  "border-rule bg-ground absolute right-2 bottom-2 cursor-grab touch-none overflow-hidden border shadow-sm",
  {
    variants: {
      leafAxis: {
        y: "h-50 w-40",
        x: "h-40 w-50",
      },
    },
  },
);

export interface MinimapProps {
  view: TreeView;
  frame: CanvasFrame;
  layers: LayersList;
}

export function Minimap({ view, frame, layers }: MinimapProps) {
  const colors = useDrawingColors();
  const { viewState, actions } = view;
  const along = frame.leafAxis === "y" ? 1 : 0;

  const { pressProps } = usePress({
    onPressStart: ({ x, y }) => {
      actions.panTo(minimapLeafAt(frame, along === 1 ? y : x));
    },
  });

  const { moveProps } = useMove({
    onMove: ({ deltaX, deltaY }) => {
      if (viewState !== undefined) {
        const delta = minimapLeafAt(frame, along === 1 ? deltaY : deltaX) - minimapLeafAt(frame, 0);

        actions.panTo(leafTarget(viewState) + delta);
      }
    },
  });

  const miniViewState = useMemo(() => minimapViewState(frame), [frame]);

  const allLayers = useMemo(
    () => [
      ...layers,
      new PolygonLayer<[number, number][]>({
        id: "minimap-window",
        data: viewState === undefined ? [] : [visibleWorldRect(viewState, frame)],
        getPolygon: (rect) => rect,
        getFillColor: withOpacity(colors.focus, WINDOW_FILL_OPACITY),
        getLineColor: colors.focus,
        getLineWidth: 1,
        lineWidthUnits: "pixels",
        stroked: true,
        filled: true,
      }),
    ],
    [layers, viewState, frame, colors],
  );

  return (
    <div aria-hidden {...mergeProps(pressProps, moveProps)} className={minimapStyle({ leafAxis: frame.leafAxis })}>
      <DeckGL views={MINIMAP_VIEW} viewState={miniViewState} controller={false} layers={allLayers} />
    </div>
  );
}
