import type { RowRange } from "./viewState";

export type ZoomKey = Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey">;

export function selectionZoomRange(
  { key, ctrlKey, metaKey, altKey, shiftKey }: ZoomKey,
  selectionRows: (() => RowRange | null) | undefined,
): RowRange | null {
  if (key !== "Enter" || ctrlKey || metaKey || altKey || shiftKey || selectionRows === undefined) {
    return null;
  }

  return selectionRows();
}
