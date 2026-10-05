import type { Palette, ThemeColors } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { mccVariables, paletteCss } from "../paletteCss";

const LIGHT: ThemeColors = {
  mcc: ["#000001", "#000002", "#000003", "#000004", "#000005", "#000006", "#000007", "#000008"],
  noMcc: "#00000a",
  ground: "#f3f5f4",
  ink: "#1f2b30",
  inkMuted: "#55656b",
  signal: "#b0265e",
  focus: "#2457c5",
  segmentA: "#3e6a8a",
  segmentB: "#8a6a3e",
};

const DARK: ThemeColors = {
  ...LIGHT,
  mcc: ["#100001", "#100002", "#100003", "#100004", "#100005", "#100006", "#100007", "#100008"],
  noMcc: "#10000a",
};

const PALETTE: Palette = { light: LIGHT, dark: DARK };

describe("palette CSS variables", () => {
  test("maps the eight slots and the no-MCC color of a theme to their variables", () => {
    expect(mccVariables(LIGHT)).toStrictEqual([
      ["--color-mcc-0", "#000001"],
      ["--color-mcc-1", "#000002"],
      ["--color-mcc-2", "#000003"],
      ["--color-mcc-3", "#000004"],
      ["--color-mcc-4", "#000005"],
      ["--color-mcc-5", "#000006"],
      ["--color-mcc-6", "#000007"],
      ["--color-mcc-7", "#000008"],
      ["--color-mcc-none", "#00000a"],
    ]);
  });

  test("writes light colors under the root and dark colors under the dark class", () => {
    expect(paletteCss(PALETTE)).toBe(
      [
        ":root {",
        "  --color-mcc-0: #000001;",
        "  --color-mcc-1: #000002;",
        "  --color-mcc-2: #000003;",
        "  --color-mcc-3: #000004;",
        "  --color-mcc-4: #000005;",
        "  --color-mcc-5: #000006;",
        "  --color-mcc-6: #000007;",
        "  --color-mcc-7: #000008;",
        "  --color-mcc-none: #00000a;",
        "}",
        ":root.dark {",
        "  --color-mcc-0: #100001;",
        "  --color-mcc-1: #100002;",
        "  --color-mcc-2: #100003;",
        "  --color-mcc-3: #100004;",
        "  --color-mcc-4: #100005;",
        "  --color-mcc-5: #100006;",
        "  --color-mcc-6: #100007;",
        "  --color-mcc-7: #100008;",
        "  --color-mcc-none: #10000a;",
        "}",
      ].join("\n"),
    );
  });
});
