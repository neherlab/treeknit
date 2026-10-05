import type { ThemeColors } from "@neherlab/treeknit-wasm";
import { useSyncExternalStore } from "react";

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

function drawingColorStore() {
  let cachedKey = "";
  let cachedColors: DrawingColors | undefined;

  return {
    snapshot(): DrawingColors {
      const style = getComputedStyle(document.documentElement);
      const key = TOKENS.map((name) => style.getPropertyValue(name)).join(";");

      if (cachedColors === undefined || key !== cachedKey) {
        cachedColors = readDrawingColors(style);
        cachedKey = key;
      }

      return cachedColors;
    },
    subscribe(onChange: () => void): () => void {
      const observer = new MutationObserver(onChange);

      observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class", "style"] });

      return () => {
        observer.disconnect();
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
  return useSyncExternalStore(subscribe, snapshot);
}
