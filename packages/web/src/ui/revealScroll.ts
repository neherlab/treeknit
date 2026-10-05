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
