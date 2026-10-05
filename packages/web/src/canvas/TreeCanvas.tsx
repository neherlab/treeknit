import { type LayersList, OrthographicView, type OrthographicViewState, type PickingInfo } from "@deck.gl/core";
import { DeckGL, type DeckGLProps, type DeckGLRef } from "@deck.gl/react";
import { cn } from "cn";
import { type MouseEvent, type ReactNode, useCallback, useId, useRef } from "react";

import { InlineNotice } from "../ui/InlineNotice";
import { Minimap } from "./Minimap";
import type { TreeView } from "./useTreeView";
import { type CanvasSize, minimapShown, type RowRange } from "./viewState";
import { browserSupportsWebGl2, WEBGL2_MISSING } from "./webgl";

export const CANVAS_DESCRIPTION = "Use the MCC table, the leaf search, and the inspector to read every value.";

const VIEW = new OrthographicView({ id: "tree", flipY: true });

const CONTROLLER = {
  doubleClickZoom: false,
  dragRotate: false,
  inertia: false,
  scrollZoom: { speed: 0.01, smooth: false },
  keyboard: { zoomSpeed: 2, moveSpeed: 48 },
};

const UNSIZED_VIEW_STATE: OrthographicViewState = { target: [0, 0], zoom: 0 };

const PICK_RADIUS_PX = 4;

type DeckEvents = Pick<DeckGLProps<OrthographicView>, "getTooltip" | "onClick" | "onHover" | "getCursor">;

export interface TreeCanvasProps extends DeckEvents {
  view: TreeView;
  label: string;
  layers: LayersList;
  minimapLayers?: LayersList;
  description?: string;
  onCladeZoom?: (info: PickingInfo) => RowRange | null;
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
  className,
  children,
  ...events
}: TreeCanvasProps) {
  const webGl2 = browserSupportsWebGl2();
  const descriptionId = useId();
  const deckRef = useRef<DeckGLRef<OrthographicView>>(null);
  const { actions, frame } = view;

  const resize = useCallback(
    (size: CanvasSize) => {
      actions.resize(size);
    },
    [actions],
  );

  const update = useCallback(
    ({ viewState }: { viewState: OrthographicViewState }) => {
      actions.update(viewState);
    },
    [actions],
  );

  const zoomToClade = useCallback(
    (event: MouseEvent<HTMLDivElement>) => {
      const deck = deckRef.current;

      if (deck === null || onCladeZoom === undefined) {
        return;
      }

      const bounds = event.currentTarget.getBoundingClientRect();
      const position = { x: event.clientX - bounds.left, y: event.clientY - bounds.top, radius: PICK_RADIUS_PX };

      void deck.pickObjectAsync(position).then((info) => {
        const range = info === null ? null : onCladeZoom(info);

        if (range !== null) {
          actions.fitRows(range);
        }

        return range;
      });
    },
    [actions, onCladeZoom],
  );

  if (!webGl2) {
    return (
      <div className={className}>
        <InlineNotice tone="warning">{WEBGL2_MISSING}</InlineNotice>
      </div>
    );
  }

  return (
    <div className={cn("relative min-h-0 min-w-0 overflow-hidden", className)}>
      {/* oxlint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- the double-click zooms to a clade; the keyboard zooms through the deck.gl controller */}
      <div
        // oxlint-disable-next-line jsx-a11y/prefer-tag-over-role -- an img element cannot hold the WebGL canvas
        role="img"
        // oxlint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- the deck.gl controller pans and zooms with the keyboard while the drawing has focus
        tabIndex={0}
        aria-label={label}
        aria-describedby={descriptionId}
        onDoubleClick={zoomToClade}
        // oxlint-disable-next-line better-tailwindcss/no-unknown-classes -- deck.gl takes the element with this class as the target of its pointer and keyboard events
        className="deck-events-root focus-visible:outline-focus absolute inset-0 outline-hidden focus-visible:outline-2 focus-visible:-outline-offset-2"
      >
        <DeckGL
          ref={deckRef}
          views={VIEW}
          viewState={view.viewState ?? UNSIZED_VIEW_STATE}
          controller={CONTROLLER}
          layers={layers}
          onResize={resize}
          onViewStateChange={update}
          {...events}
        />
      </div>
      <p id={descriptionId} className="sr-only">
        {description}
      </p>
      {minimapLayers !== undefined && frame !== undefined && minimapShown(frame) ? (
        <Minimap view={view} frame={frame} layers={minimapLayers} />
      ) : null}
      {children}
    </div>
  );
}
