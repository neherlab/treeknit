import { describe, expect, test } from "vitest";

import { parseColor, withOpacity } from "../color";
import { type CustomProperties, MCC_SLOT_COUNT, mccColor, nextColorReading, readDrawingColors } from "../drawingColors";

describe("parseColor", () => {
  test.each([
    ["#f3f5f4", [243, 245, 244, 255]],
    ["#B0265E", [176, 38, 94, 255]],
    ["  #1f2b30\n", [31, 43, 48, 255]],
    ["#abc", [170, 187, 204, 255]],
    ["#abc8", [170, 187, 204, 136]],
    ["#2457c580", [36, 87, 197, 128]],
    ["rgb(36, 87, 197)", [36, 87, 197, 255]],
    ["rgba(36, 87, 197, 0.5)", [36, 87, 197, 128]],
    ["rgb(36 87 197 / 25%)", [36, 87, 197, 64]],
    ["rgb(100% 0% 50%)", [255, 0, 128, 255]],
  ])("converts %s to RGBA bytes", (text, expected) => {
    expect(parseColor(text)).toStrictEqual(expected);
  });

  test.each(["", "red", "#12345", "#ggg", "rgb(1, 2)", "rgb(a, b, c)", "hsl(0 0% 0%)", "oklch(0.5 0.1 200)"])(
    "rejects %o",
    (text) => {
      expect(parseColor(text)).toBeUndefined();
    },
  );
});

describe("withOpacity", () => {
  test("scales the alpha byte and clamps the factor", () => {
    expect(withOpacity([10, 20, 30, 255], 0.55)).toStrictEqual([10, 20, 30, 140]);
    expect(withOpacity([10, 20, 30, 128], 0.25)).toStrictEqual([10, 20, 30, 32]);
    expect(withOpacity([10, 20, 30, 255], 2)).toStrictEqual([10, 20, 30, 255]);
    expect(withOpacity([10, 20, 30, 255], -1)).toStrictEqual([10, 20, 30, 0]);
  });
});

function tokens(values: Record<string, string>): CustomProperties {
  return {
    getPropertyValue: (name) => values[name] ?? "",
  };
}

const LIGHT = {
  "--color-ground": "#f3f5f4",
  "--color-ink": "#1f2b30",
  "--color-ink-muted": "#55656b",
  "--color-signal": "#b0265e",
  "--color-focus": "#2457c5",
  "--color-segment-a": "#3e6a8a",
  "--color-segment-b": "#8a6a3e",
  "--color-mcc-0": "#2f4b9a",
  "--color-mcc-1": "#c9a227",
  "--color-mcc-2": "#2c8c83",
  "--color-mcc-3": "#5e3c82",
  "--color-mcc-4": "#7a5537",
  "--color-mcc-5": "#7d8f3a",
  "--color-mcc-6": "#d07a1e",
  "--color-mcc-7": "#5b7c99",
  "--color-mcc-none": "#9aa5a2",
};

describe("readDrawingColors", () => {
  test("reads every drawing token as RGBA", () => {
    const colors = readDrawingColors(tokens(LIGHT));

    expect(colors.ground).toStrictEqual([243, 245, 244, 255]);
    expect(colors.segmentB).toStrictEqual([138, 106, 62, 255]);
    expect(colors.mcc).toHaveLength(8);
    expect(colors.mcc[7]).toStrictEqual([91, 124, 153, 255]);
    expect(colors.mccNone).toStrictEqual([154, 165, 162, 255]);
  });

  test("names the token that holds no color", () => {
    expect(() => readDrawingColors(tokens({ ...LIGHT, "--color-mcc-3": "" }))).toThrow("--color-mcc-3");
  });
});

describe("mccColor", () => {
  const colors = readDrawingColors(tokens(LIGHT));

  test("returns the slot color, and the no-MCC color without a slot", () => {
    expect(mccColor(colors, 2)).toStrictEqual([44, 140, 131, 255]);
    expect(mccColor(colors, null)).toStrictEqual(colors.mccNone);
    expect(mccColor(colors, undefined)).toStrictEqual(colors.mccNone);
  });

  test("returns the last of the eight palette slots", () => {
    expect(mccColor(colors, MCC_SLOT_COUNT - 1)).toStrictEqual(colors.mcc.at(-1));
  });

  test.each([MCC_SLOT_COUNT, -1, 1.5])("rejects slot %d, which breaks the palette contract", (slot) => {
    expect(() => mccColor(colors, slot)).toThrow(RangeError);
  });
});

describe("nextColorReading", () => {
  const first = nextColorReading(undefined, tokens(LIGHT));

  test("keeps the same reading while the tokens are unchanged, so consumers do not render again", () => {
    expect(nextColorReading(first, tokens({ ...LIGHT }))).toBe(first);
  });

  test("reads the colors again when a token changes", () => {
    const next = nextColorReading(first, tokens({ ...LIGHT, "--color-mcc-0": "#000000" }));

    expect(next.colors.mcc[0]).toStrictEqual([0, 0, 0, 255]);
    expect(next.colors.ground).toStrictEqual(first.colors.ground);
  });
});
