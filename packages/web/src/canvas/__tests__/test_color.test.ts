import type { ThemeColors } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { cssColor, withOpacity } from "../color";
import { type DrawingColors, MCC_SLOT_COUNT, mccColor, paletteTheme } from "../drawingColors";

describe("cssColor", () => {
  test("writes an RGBA color as a CSS rgb() color with alpha", () => {
    expect([cssColor([47, 75, 154, 255]), cssColor([47, 75, 154, 140])]).toStrictEqual([
      "rgb(47 75 154 / 1)",
      "rgb(47 75 154 / 0.549)",
    ]);
  });
});

describe("withOpacity", () => {
  test("scales the alpha byte and clamps the factor", () => {
    expect(withOpacity([10, 20, 30, 255], 0.55)).toStrictEqual([10, 20, 30, 140]);
    expect(withOpacity([10, 20, 30, 128], 0.25)).toStrictEqual([10, 20, 30, 32]);
    expect(withOpacity([10, 20, 30, 255], 2)).toStrictEqual([10, 20, 30, 255]);
    expect(withOpacity([10, 20, 30, 255], -1)).toStrictEqual([10, 20, 30, 0]);
  });
});

const LIGHT: DrawingColors = {
  mcc: [
    [47, 75, 154, 255],
    [201, 162, 39, 255],
    [44, 140, 131, 255],
    [94, 60, 130, 255],
    [122, 85, 55, 255],
    [125, 143, 58, 255],
    [208, 122, 30, 255],
    [91, 124, 153, 255],
  ],
  noMcc: [154, 165, 162, 255],
  ground: [243, 245, 244, 255],
  ink: [31, 43, 48, 255],
  inkMuted: [85, 101, 107, 255],
  signal: [176, 38, 94, 255],
  focus: [36, 87, 197, 255],
  segmentA: [62, 106, 138, 255],
  segmentB: [138, 106, 62, 255],
};

const DARK: DrawingColors = { ...LIGHT, ground: [22, 32, 36, 255], ink: [220, 228, 225, 255] };

describe("paletteTheme", () => {
  test.each([
    ["dark", DARK],
    ["light", LIGHT],
    ["system", LIGHT],
    [undefined, LIGHT],
  ])("takes the RGBA colors of the palette theme for the resolved theme %o", (resolved, expected) => {
    const palette = { light: TEXT, dark: TEXT, lightRgba: LIGHT, darkRgba: DARK };

    expect(paletteTheme(palette, resolved)).toBe(expected);
  });
});

describe("mccColor", () => {
  const colors = LIGHT;

  test("returns the slot color, and the no-MCC color without a slot", () => {
    expect(mccColor(colors, 2)).toStrictEqual([44, 140, 131, 255]);
    expect(mccColor(colors, null)).toStrictEqual(colors.noMcc);
    expect(mccColor(colors, undefined)).toStrictEqual(colors.noMcc);
  });

  test("returns the last of the eight palette slots", () => {
    expect(mccColor(colors, MCC_SLOT_COUNT - 1)).toStrictEqual(colors.mcc.at(-1));
  });

  test.each([MCC_SLOT_COUNT, -1, 1.5])("rejects slot %d, which breaks the palette contract", (slot) => {
    expect(() => mccColor(colors, slot)).toThrow(RangeError);
  });
});

const TEXT: ThemeColors<string> = {
  mcc: ["#000001", "#000002", "#000003", "#000004", "#000005", "#000006", "#000007", "#000008"],
  noMcc: "#000009",
  ground: "#162024",
  ink: "#dce4e1",
  inkMuted: "#9aaaa6",
  signal: "#e0619a",
  focus: "#7da2f0",
  segmentA: "#406e8f",
  segmentB: "#8a6a3e",
};
