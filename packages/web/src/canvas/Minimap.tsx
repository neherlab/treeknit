import { type LayersList, OrthographicView } from "@deck.gl/core";
import { PolygonLayer } from "@deck.gl/layers";
import { DeckGL } from "@deck.gl/react";
import { cn } from "cn";
import { useCallback, useMemo, useState } from "react";
import { mergeProps, useMove, usePress } from "react-aria";
import { useErrorBoundary } from "react-error-boundary";

import { withOpacity } from "./color";
import { useDrawingColors } from "./drawingColors";
import type { TreeView } from "./useTreeView";
import {
  type CanvasFrame,
  type CanvasSize,
  type LeafAxis,
  minimapLeafAt,
  minimapLeafOffset,
  minimapSize,
  minimapViewState,
  visibleWorldRect,
} from "./viewState";

const MINIMAP_VIEW = new OrthographicView({ id: "minimap", flipY: true });

const WINDOW_FILL_OPACITY = 0.12;

const IN_FLOW: Partial<CSSStyleDeclaration> = { position: "relative" };

export interface MinimapProps {
  view: TreeView;
  frame: CanvasFrame;
  layers: LayersList;
}

export function Minimap({ view, frame, layers }: MinimapProps) {
  const colors = useDrawingColors();
  const { showBoundary } = useErrorBoundary();
  const { viewState, actions } = view;
  const { leafAxis } = frame;
  const along = leafAxis === "y" ? 1 : 0;
  const planned = useMemo(() => minimapSize(frame), [frame]);
  const [measured, setMeasured] = useState<{ leafAxis: LeafAxis; size: CanvasSize } | undefined>(undefined);
  const size = measured?.leafAxis === leafAxis ? measured.size : planned;

  const measure = useCallback(
    (next: CanvasSize) => {
      setMeasured({ leafAxis, size: next });
    },
    [leafAxis],
  );

  const { pressProps } = usePress({
    onPressStart: ({ x, y }) => {
      actions.panTo(minimapLeafAt(frame, size, along === 1 ? y : x));
    },
  });

  const { moveProps } = useMove({
    onMove: ({ deltaX, deltaY }) => {
      actions.panBy(minimapLeafOffset(frame, size, along === 1 ? deltaY : deltaX));
    },
  });

  const miniViewState = useMemo(() => minimapViewState(frame, size), [frame, size]);

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
    <div
      aria-hidden
      {...mergeProps(pressProps, moveProps)}
      className={cn(
        "border-rule bg-ground relative shrink-0 cursor-grab touch-none overflow-hidden",
        along === 1 ? "border-l" : "border-t",
      )}
    >
      <DeckGL
        views={MINIMAP_VIEW}
        viewState={miniViewState}
        controller={false}
        layers={allLayers}
        width={planned.width}
        height={planned.height}
        style={IN_FLOW}
        onResize={measure}
        onError={showBoundary}
      />
    </div>
  );
}
