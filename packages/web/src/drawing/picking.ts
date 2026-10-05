import type { PickingInfo } from "@deck.gl/core";
import { useCallback, useMemo } from "react";

import type { RowRange } from "../canvas/viewState";
import type { Selection } from "./selection";
import { type TooltipContent, tooltipContent } from "./tooltipContent";

export type LayerPicks<L extends string, G, T> = Readonly<Record<L, (geometry: G, index: number) => T | undefined>>;

export interface PickRules<D, G, T> {
  targetAt(geometry: G, layer: string, index: number): T | undefined;
  tooltip(data: D, target: T): string[];
  clickSelection(data: D, target: T | undefined): Selection;
  targetRows(data: D, target: T): RowRange | null;
  selectionRows(data: D, selection: Selection): RowRange | null;
}

export interface DrawingPicking {
  getTooltip: (info: PickingInfo) => TooltipContent | null;
  onClick: (info: PickingInfo) => void;
  onCladeZoom: (info: PickingInfo) => RowRange | null;
  onSelectionZoom: () => RowRange | null;
}

export function pickTarget<L extends string, G, T>(
  picks: LayerPicks<L, G, T>,
  geometry: G,
  layer: string,
  index: number,
): T | undefined {
  const pick = Object.entries<(geometry: G, index: number) => T | undefined>(picks).find(([id]) => id === layer)?.[1];

  return pick?.(geometry, index);
}

export function useDrawingPicking<D, G, T>(
  rules: PickRules<D, G, T>,
  data: D,
  geometry: G,
  selection: Selection,
  onSelect: (selection: Selection) => void,
): DrawingPicking {
  const handlers = useMemo(() => {
    const targetOf = (info: PickingInfo) =>
      info.layer === null || info.layer === undefined ? undefined : rules.targetAt(geometry, info.layer.id, info.index);

    return {
      getTooltip: (info: PickingInfo) => {
        const target = targetOf(info);

        return target === undefined ? null : tooltipContent(rules.tooltip(data, target));
      },
      onClick: (info: PickingInfo) => {
        onSelect(rules.clickSelection(data, targetOf(info)));
      },
      onCladeZoom: (info: PickingInfo) => {
        const target = targetOf(info);

        return target === undefined ? null : rules.targetRows(data, target);
      },
    };
  }, [rules, data, geometry, onSelect]);

  const onSelectionZoom = useCallback(() => rules.selectionRows(data, selection), [rules, data, selection]);

  return useMemo(() => ({ ...handlers, onSelectionZoom }), [handlers, onSelectionZoom]);
}
