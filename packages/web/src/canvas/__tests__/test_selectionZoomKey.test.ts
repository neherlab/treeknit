import { describe, expect, test } from "vitest";

import { selectionZoomRange, type ZoomKey } from "../selectionZoomKey";
import type { RowRange } from "../viewState";

const SELECTED: RowRange = { first: 3, last: 7 };

const PLAIN: ZoomKey = { key: "Enter", ctrlKey: false, metaKey: false, altKey: false, shiftKey: false };

function selected(): RowRange {
  return SELECTED;
}

function nothingSelected(): null {
  return null;
}

describe("selectionZoomRange", () => {
  test("frames the selection on plain Enter", () => {
    expect(selectionZoomRange(PLAIN, selected)).toStrictEqual(SELECTED);
  });

  test.each([
    ["Ctrl+Enter", { ...PLAIN, ctrlKey: true }],
    ["Cmd+Enter", { ...PLAIN, metaKey: true }],
    ["Alt+Enter", { ...PLAIN, altKey: true }],
    ["Shift+Enter", { ...PLAIN, shiftKey: true }],
    ["Space", { ...PLAIN, key: " " }],
  ])("leaves %s to the page", (_name, key) => {
    expect(selectionZoomRange(key, selected)).toBeNull();
  });

  test("leaves Enter to the page without a selection or a selection zoom", () => {
    expect([selectionZoomRange(PLAIN, nothingSelected), selectionZoomRange(PLAIN, undefined)]).toStrictEqual([
      null,
      null,
    ]);
  });
});
