export interface Span {
  start: number;
  end: number;
}

export function revealScroll(item: Span, view: Span): number {
  if (item.start < view.start) {
    return item.start - view.start;
  }

  if (item.end > view.end) {
    return Math.min(item.end - view.end, item.start - view.start);
  }

  return 0;
}

export interface Box {
  top: number;
  left: number;
  width: number;
  height: number;
}

export interface Scroller {
  offsetWidth: number;
  offsetHeight: number;
  clientTop: number;
  clientLeft: number;
  clientWidth: number;
  clientHeight: number;
}

export interface ScrollOffset {
  top: number;
  left: number;
}

export function revealOffset(item: Box, view: Box, scroller: Scroller): ScrollOffset {
  const scaleX = scroller.offsetWidth > 0 ? view.width / scroller.offsetWidth : 1;
  const scaleY = scroller.offsetHeight > 0 ? view.height / scroller.offsetHeight : 1;
  const top = (item.top - view.top) / scaleY - scroller.clientTop;
  const left = (item.left - view.left) / scaleX - scroller.clientLeft;

  return {
    top: revealScroll({ start: top, end: top + item.height / scaleY }, { start: 0, end: scroller.clientHeight }),
    left: revealScroll({ start: left, end: left + item.width / scaleX }, { start: 0, end: scroller.clientWidth }),
  };
}
