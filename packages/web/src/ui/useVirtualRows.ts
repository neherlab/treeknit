import { useVirtualizer, type VirtualItem } from "@tanstack/react-virtual";
import { type RefObject, useCallback } from "react";

export const TABLE_ROW_PX = 32;

const OVERSCAN_ROWS = 12;

export interface VirtualRows {
  items: VirtualItem[];
  before: number;
  after: number;
}

export function useVirtualRows(count: number, scrollRef: RefObject<HTMLElement | null>): VirtualRows {
  const estimateSize = useCallback(() => TABLE_ROW_PX, []);
  const getScrollElement = useCallback(() => scrollRef.current, [scrollRef]);

  // oxlint-disable-next-line react/incompatible-library -- TanStack Virtual is the owner of virtual lists; the app does not run the React Compiler, which this warning is about
  const virtualizer = useVirtualizer({ count, getScrollElement, estimateSize, overscan: OVERSCAN_ROWS });
  const items = virtualizer.getVirtualItems();

  return { items, ...virtualGaps(items, virtualizer.getTotalSize()) };
}

export function virtualGaps(items: readonly Pick<VirtualItem, "start" | "end">[], totalPx: number) {
  return { before: items[0]?.start ?? 0, after: Math.max(totalPx - (items.at(-1)?.end ?? 0), 0) };
}
