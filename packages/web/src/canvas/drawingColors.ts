import type { ColorRole, Palette, Rgba, ThemeColors } from "@neherlab/treeknit-wasm";
import { useTheme } from "next-themes";

import { useSuspensePalette } from "../analysis/queries";

export const MCC_SLOT_COUNT: DrawingColors["mcc"]["length"] = 8;

export type DrawingColors = ThemeColors<Rgba>;

const SLOT_ROLES: ReadonlySet<ColorRole> = new Set(["mcc", "noMcc"]);

export function mccColor(colors: DrawingColors, slot: number | null | undefined): Rgba {
  if (slot === null || slot === undefined) {
    return colors.noMcc;
  }

  const color = colors.mcc[slot];

  if (color === undefined) {
    throw new RangeError(`MCC color slot ${String(slot)} is outside the ${String(colors.mcc.length)} palette slots`);
  }

  return color;
}

export function roleColor(colors: DrawingColors, role: ColorRole, slot: number | null): Rgba {
  return role === "mcc" ? mccColor(colors, slot) : colors[role];
}

export function roleColorByMcc(colors: DrawingColors, role: ColorRole, slot: number | null, colorByMcc: boolean): Rgba {
  return SLOT_ROLES.has(role) && !colorByMcc ? colors.inkMuted : roleColor(colors, role, slot);
}

export function paletteTheme(palette: Palette, resolvedTheme: string | undefined): DrawingColors {
  return resolvedTheme === "dark" ? palette.darkRgba : palette.lightRgba;
}

export function useDrawingColors(): DrawingColors {
  const palette = useSuspensePalette();
  const { resolvedTheme } = useTheme();

  return paletteTheme(palette, resolvedTheme);
}
