import type { Palette, ThemeColors } from "@neherlab/treeknit-wasm";
import { useTheme } from "next-themes";
import { useMemo } from "react";

import { useSuspensePalette } from "../analysis/queries";
import { parseColor, type Rgba } from "./color";

export const MCC_SLOT_COUNT: ThemeColors["mcc"]["length"] = 8;

export interface DrawingColors {
  ground: Rgba;
  ink: Rgba;
  inkMuted: Rgba;
  signal: Rgba;
  focus: Rgba;
  segmentA: Rgba;
  segmentB: Rgba;
  mcc: readonly Rgba[];
  mccNone: Rgba;
}

export function paletteDrawingColors(theme: ThemeColors): DrawingColors {
  return {
    ground: paletteColor("ground", theme.ground),
    ink: paletteColor("ink", theme.ink),
    inkMuted: paletteColor("inkMuted", theme.inkMuted),
    signal: paletteColor("signal", theme.signal),
    focus: paletteColor("focus", theme.focus),
    segmentA: paletteColor("segmentA", theme.segmentA),
    segmentB: paletteColor("segmentB", theme.segmentB),
    mcc: theme.mcc.map((color, slot) => paletteColor(`mcc[${String(slot)}]`, color)),
    mccNone: paletteColor("noMcc", theme.noMcc),
  };
}

function paletteColor(name: string, value: string): Rgba {
  const color = parseColor(value);

  if (color === undefined) {
    throw new Error(`The palette color ${name} is "${value}", which is not a hex or rgb() color`);
  }

  return color;
}

export function mccColor(colors: DrawingColors, slot: number | null | undefined): Rgba {
  if (slot === null || slot === undefined) {
    return colors.mccNone;
  }

  const color = colors.mcc[slot];

  if (color === undefined) {
    throw new RangeError(`MCC color slot ${String(slot)} is outside the ${String(colors.mcc.length)} palette slots`);
  }

  return color;
}

export function paletteTheme(palette: Palette, resolvedTheme: string | undefined): ThemeColors {
  return resolvedTheme === "dark" ? palette.dark : palette.light;
}

export function useDrawingColors(): DrawingColors {
  const palette = useSuspensePalette();
  const { resolvedTheme } = useTheme();
  const theme = paletteTheme(palette, resolvedTheme);

  return useMemo(() => paletteDrawingColors(theme), [theme]);
}
