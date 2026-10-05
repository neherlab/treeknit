import { type LayersList, OrthographicView, type OrthographicViewState, type PickingInfo } from "@deck.gl/core";
import { DeckGL, type DeckGLProps, type DeckGLRef } from "@deck.gl/react";
import { cn } from "cn";
import { type MouseEvent, type ReactNode, type RefObject, useCallback, useId, useMemo, useRef, useState } from "react";
import { useKeyboard } from "react-aria";
import { useErrorBoundary } from "react-error-boundary";

import { InlineNotice } from "../ui/InlineNotice";
import { Minimap } from "./Minimap";
import { selectionZoomRange } from "./selectionZoomKey";
import { useMeasuredSize } from "./useMeasuredSize";
import type { TreeView, TreeViewActions } from "./useTreeView";
import { type CanvasFrame, type MeasuredSize, minimapShown, type RowRange, type TreeViewState } from "./viewState";
import { browserSupportsWebGl2, WEBGL2_MISSING } from "./webgl";

export const CANVAS_DESCRIPTION =
  "Arrow keys pan, plus and minus zoom, Enter zooms to the selected clade. Use the MCC table, the leaf search, and the inspector to read every value.";

const VIEW_ID = "tree";

const CONTROLLER = {
  doubleClickZoom: false,
  dragRotate: false,
  inertia: false,
  scrollZoom: { speed: 0.01, smooth: false },
  keyboard: { zoomSpeed: 2, moveSpeed: 48 },
};

const PICK_RADIUS_PX = 4;

type PickPosition = Parameters<DeckGLRef<OrthographicView>["pickObjectAsync"]>[0];

type DeckEvents = Pick<DeckGLProps<OrthographicView>, "getTooltip" | "onClick" | "onHover" | "getCursor">;

export interface TreeCanvasProps extends DeckEvents {
  view: TreeView;
  label: string;
  layers: LayersList;
  minimapLayers?: LayersList;
  description?: string;
  onCladeZoom?: (info: PickingInfo) => RowRange | null;
  onSelectionZoom?: () => RowRange | null;
  className?: string;
  children?: ReactNode;
}

export function TreeCanvas({
  view,
  label,
  layers,
  minimapLayers,
  description = CANVAS_DESCRIPTION,
  onCladeZoom,
  onSelectionZoom,
  className,
  children,
  ...events
}: TreeCanvasProps) {
  const [webGl2] = useState(browserSupportsWebGl2);
  const descriptionId = useId();
  const deckRef = useRef<DeckGLRef<OrthographicView>>(null);
  const { actions, frame } = view;
  const { showBoundary } = useErrorBoundary();

  const resize = useCallback(
    (measured: MeasuredSize) => {
      actions.resize(measured);
    },
    [actions],
  );

  const { areaRef, canvasRef } = useMeasuredSize(resize);

  const zoomToCladeAt = useCallback(
    async (position: PickPosition) => {
      const deck = deckRef.current;

      if (deck === null || onCladeZoom === undefined) {
        return;
      }

      try {
        const info = await deck.pickObjectAsync(position);
        const range = info === null ? null : onCladeZoom(info);

        if (range !== null) {
          actions.fitRows(range);
        }
      } catch (error) {
        showBoundary(error);
      }
    },
    [actions, onCladeZoom, showBoundary],
  );

  const zoomToClade = useCallback(
    (event: MouseEvent<HTMLDivElement>) => {
      const bounds = event.currentTarget.getBoundingClientRect();

      void zoomToCladeAt({ x: event.clientX - bounds.left, y: event.clientY - bounds.top, radius: PICK_RADIUS_PX });
    },
    [zoomToCladeAt],
  );

  const { keyboardProps } = useKeyboard({
    onKeyDown: (event) => {
      const range = selectionZoomRange(event, onSelectionZoom);

      if (range === null) {
        event.continuePropagation();
      } else {
        actions.fitRows(range);
      }
    },
  });

  if (!webGl2) {
    return (
      <div className={className}>
        <InlineNotice tone="warning">{WEBGL2_MISSING}</InlineNotice>
      </div>
    );
  }

  return (
    <div ref={areaRef} className={cn("flex min-h-0 min-w-0 flex-col overflow-hidden", className)}>
      {children}
      <div className={cn("flex min-h-0 flex-1", frame?.leafAxis === "x" ? "flex-col" : "flex-row")}>
        <div ref={canvasRef} className="relative min-h-0 min-w-0 flex-1">
          {/* oxlint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- jsx-a11y counts the application role as non-interactive, but it is the ARIA role of an element that handles its own pointer and keys; the double-click zooms to a clade */}
          <div
            role="application"
            aria-roledescription="tree drawing"
            // oxlint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- jsx-a11y counts the application role as non-interactive; the deck.gl controller pans and zooms with the keyboard while the drawing has focus
            tabIndex={0}
            aria-label={label}
            aria-describedby={descriptionId}
            onDoubleClick={zoomToClade}
            {...keyboardProps}
            // oxlint-disable-next-line better-tailwindcss/no-unknown-classes -- deck.gl takes the element with this class as the target of its pointer and keyboard events
            className="deck-events-root focus-visible:outline-focus absolute inset-0 outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2"
          >
            {frame === undefined || view.viewState === undefined ? null : (
              <TreeDeck
                deckRef={deckRef}
                frame={frame}
                viewState={view.viewState}
                layers={layers}
                actions={actions}
                onError={showBoundary}
                events={events}
              />
            )}
          </div>
          <p id={descriptionId} className="sr-only">
            {description}
          </p>
        </div>
        {minimapLayers !== undefined && frame !== undefined && minimapShown(frame) ? (
          <Minimap view={view} frame={frame} layers={minimapLayers} />
        ) : null}
      </div>
    </div>
  );
}

function TreeDeck({ deckRef, frame, viewState, layers, actions, onError, events }: TreeDeckProps) {
  const { width, height } = frame.size;
  const views = useMemo(() => new OrthographicView({ id: VIEW_ID, flipY: true, width, height }), [width, height]);

  const update = useCallback(
    ({ viewState: next }: { viewState: OrthographicViewState }) => {
      actions.update(next);
    },
    [actions],
  );

  return (
    <DeckGL
      ref={deckRef}
      views={views}
      viewState={viewState}
      controller={CONTROLLER}
      layers={layers}
      onViewStateChange={update}
      onError={onError}
      {...events}
    />
  );
}

interface TreeDeckProps {
  deckRef: RefObject<DeckGLRef<OrthographicView> | null>;
  frame: CanvasFrame;
  viewState: TreeViewState;
  layers: LayersList;
  actions: TreeViewActions;
  onError: (error: Error) => void;
  events: DeckEvents;
}
