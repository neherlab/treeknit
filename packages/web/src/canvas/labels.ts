import { useSyncExternalStore } from "react";

export type LabelMode = "auto" | "on" | "off";

export const LABEL_FONT_FAMILY = '"IBM Plex Sans Condensed", "IBM Plex Sans", sans-serif';

export const LABEL_FONT_WEIGHT = 400;

export const LABEL_FONT_SIZE_PX = 12;

export const LABEL_AUTO_MIN_ROW_PX = 10;

export const LABEL_MAX_LENGTH = 40;

const ELLIPSIS = "…";

const graphemes = new Intl.Segmenter(undefined, { granularity: "grapheme" });

export function labelsVisible(mode: LabelMode, rowPx: number): boolean {
  return mode === "on" || (mode === "auto" && rowPx >= LABEL_AUTO_MIN_ROW_PX);
}

export function shortenLabel(name: string, maxLength = LABEL_MAX_LENGTH): string {
  const parts = Array.from(graphemes.segment(name), ({ segment }) => segment);

  if (parts.length <= maxLength) {
    return name;
  }

  const kept = maxLength - 1;
  const head = Math.ceil(kept / 2);

  return parts.slice(0, head).join("") + ELLIPSIS + parts.slice(parts.length - (kept - head)).join("");
}

export const LABEL_FONT = `${String(LABEL_FONT_WEIGHT)} ${String(LABEL_FONT_SIZE_PX)}px ${LABEL_FONT_FAMILY}`;

export interface FontLoader {
  load(font: string): Promise<void>;
  reportFailure(font: string, error: Error): void;
}

export interface FontStore {
  subscribe(onChange: () => void): () => void;
  isReady(): boolean;
}

export function labelFontStore(loader: FontLoader): FontStore {
  let ready = false;
  let loading: Promise<void> | undefined;
  const listeners = new Set<() => void>();

  async function waitForFont(): Promise<void> {
    try {
      await loader.load(LABEL_FONT);
    } catch (error) {
      loader.reportFailure(LABEL_FONT, error instanceof Error ? error : new Error(String(error)));
    }

    ready = true;

    for (const listener of listeners) {
      listener();
    }
  }

  return {
    subscribe(onChange) {
      listeners.add(onChange);
      loading ??= waitForFont();

      return () => {
        listeners.delete(onChange);
      };
    },
    isReady() {
      return ready;
    },
  };
}

const fontStore = labelFontStore({
  load: async (font) => {
    await document.fonts.load(font);
  },
  reportFailure: (font, error) => {
    console.error(`The label font "${font}" did not load, so labels use a fallback font`, error);
  },
});

function subscribeFont(onChange: () => void): () => void {
  return fontStore.subscribe(onChange);
}

function fontReady(): boolean {
  return fontStore.isReady();
}

function fontNotReadyOnServer(): boolean {
  return false;
}

export function useLabelFontReady(): boolean {
  return useSyncExternalStore(subscribeFont, fontReady, fontNotReadyOnServer);
}
