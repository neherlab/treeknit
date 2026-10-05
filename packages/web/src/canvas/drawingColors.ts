import type { ThemeColors } from "@neherlab/treeknit-wasm";
import { useTheme } from "next-themes";
import { useMemo, useSyncExternalStore } from "react";

import { usePalette } from "../analysis/queries";
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

export interface CustomProperties {
  getPropertyValue(name: string): string;
}

const NAMED_TOKENS = {
  ground: "--color-ground",
  ink: "--color-ink",
  inkMuted: "--color-ink-muted",
  signal: "--color-signal",
  focus: "--color-focus",
  segmentA: "--color-segment-a",
  segmentB: "--color-segment-b",
  mccNone: "--color-mcc-none",
} as const satisfies Record<Exclude<keyof DrawingColors, "mcc">, string>;

const MCC_TOKENS = Array.from({ length: MCC_SLOT_COUNT }, (_, slot) => `--color-mcc-${String(slot)}`);

const TOKENS = [...Object.values(NAMED_TOKENS), ...MCC_TOKENS];

function readToken(style: CustomProperties, name: string): Rgba {
  const value = style.getPropertyValue(name);
  const color = parseColor(value);

  if (color === undefined) {
    throw new Error(`The theme token ${name} holds "${value}", which is not a hex or rgb() color`);
  }

  return color;
}

export function readDrawingColors(style: CustomProperties): DrawingColors {
  return {
    ground: readToken(style, NAMED_TOKENS.ground),
    ink: readToken(style, NAMED_TOKENS.ink),
    inkMuted: readToken(style, NAMED_TOKENS.inkMuted),
    signal: readToken(style, NAMED_TOKENS.signal),
    focus: readToken(style, NAMED_TOKENS.focus),
    segmentA: readToken(style, NAMED_TOKENS.segmentA),
    segmentB: readToken(style, NAMED_TOKENS.segmentB),
    mcc: MCC_TOKENS.map((name) => readToken(style, name)),
    mccNone: readToken(style, NAMED_TOKENS.mccNone),
  };
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

export interface ColorReading {
  key: string;
  colors: DrawingColors;
}

export function nextColorReading(previous: ColorReading | undefined, style: CustomProperties): ColorReading {
  const key = TOKENS.map((name) => style.getPropertyValue(name)).join(";");

  return previous?.key === key ? previous : { key, colors: readDrawingColors(style) };
}

function drawingColorStore() {
  let reading: ColorReading | undefined;
  let failure: { error: unknown } | undefined;
  let observer: MutationObserver | undefined;
  const listeners = new Set<() => void>();

  function read(): ColorReading {
    reading = nextColorReading(reading, getComputedStyle(document.documentElement));

    return reading;
  }

  function notify() {
    for (const listener of listeners) {
      listener();
    }
  }

  function refresh() {
    const previous = reading;

    try {
      if (read() === previous && failure === undefined) {
        return;
      }

      failure = undefined;
    } catch (error) {
      failure = { error };
    }

    notify();
  }

  return {
    snapshot(): DrawingColors {
      if (failure !== undefined) {
        throw failure.error;
      }

      return (observer === undefined ? read() : (reading ?? read())).colors;
    },
    subscribe(onChange: () => void): () => void {
      listeners.add(onChange);

      if (observer === undefined) {
        failure = undefined;
        read();
        observer = new MutationObserver(refresh);
        observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class", "style"] });
      }

      return () => {
        listeners.delete(onChange);

        if (listeners.size === 0) {
          observer?.disconnect();
          observer = undefined;
        }
      };
    },
  };
}

const store = drawingColorStore();

function subscribe(onChange: () => void): () => void {
  return store.subscribe(onChange);
}

function snapshot(): DrawingColors {
  return store.snapshot();
}

export function useDrawingColors(): DrawingColors {
  const palette = usePalette();
  const { resolvedTheme } = useTheme();
  const tokenColors = useSyncExternalStore(subscribe, snapshot);
  const theme = palette.data === undefined ? undefined : palette.data[resolvedTheme === "dark" ? "dark" : "light"];
  const themeColors = useMemo(() => (theme === undefined ? undefined : paletteDrawingColors(theme)), [theme]);

  if (palette.isError) {
    throw palette.error;
  }

  return themeColors ?? tokenColors;
}
