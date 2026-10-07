import { describe, expect, test } from "vitest";

import { PANE_WIDTHS } from "../paneWidths";

describe("pane widths", () => {
  test.each(Object.entries(PANE_WIDTHS))("sizes a pane of %s px by its width in pixels", (widthPx, widthClass) => {
    expect(widthClass).toBe(`w-[${widthPx}px]`);
  });
});
