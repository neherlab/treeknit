import { useVirtualizer, type VirtualItem } from "@tanstack/react-virtual";
import { type RefObject, useCallback, useState } from "react";

const ESTIMATED_ROW_PX = 32;

const OVERSCAN_ROWS = 12;

export interface VirtualRows {
  items: VirtualItem[];
  before: number;
  after: number;
  measureRow: (row: HTMLTableRowElement | null) => void;
  headerRef: (header: HTMLElement | null) => void;
}

export function useVirtualRows(count: number, scrollRef: RefObject<HTMLElement | null>): VirtualRows {
  const [headerPx, setHeaderPx] = useState(0);
  const estimateSize = useCallback(() => ESTIMATED_ROW_PX, []);
  const getScrollElement = useCallback(() => scrollRef.current, [scrollRef]);

  const headerRef = useCallback((header: HTMLElement | null) => {
    setHeaderPx(header?.offsetHeight ?? 0);
  }, []);

  // oxlint-disable-next-line react/incompatible-library -- TanStack Virtual is the owner of virtual lists; the app does not run the React Compiler, which this warning is about
  const virtualizer = useVirtualizer({
    count,
    getScrollElement,
    estimateSize,
    overscan: OVERSCAN_ROWS,
    scrollMargin: headerPx,
  });

  const items = virtualizer.getVirtualItems();

  return {
    items,
    ...virtualGaps(items, virtualizer.getTotalSize(), headerPx),
    measureRow: virtualizer.measureElement,
    headerRef,
  };
}

export function virtualGaps(
  items: readonly Pick<VirtualItem, "start" | "end">[],
  totalPx: number,
  marginPx: number,
): Pick<VirtualRows, "before" | "after"> {
  const first = items[0];
  const last = items.at(-1);

  return {
    before: first === undefined ? 0 : first.start - marginPx,
    after: last === undefined ? 0 : Math.max(totalPx - (last.end - marginPx), 0),
  };
}
