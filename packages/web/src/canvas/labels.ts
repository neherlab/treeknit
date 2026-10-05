import { useCallback, useSyncExternalStore } from "react";

export type LabelMode = "auto" | "on" | "off";

export const LABEL_FONT_FAMILY = '"IBM Plex Sans Condensed", "IBM Plex Sans", sans-serif';

export const LABEL_FONT_WEIGHT = 400;

export const LABEL_FONT_SIZE_PX = 12;

export const LABEL_AUTO_MIN_ROW_PX = 10;

export const LABEL_MAX_LENGTH = 40;

export const RIBBON_MAX_ROW_PX = 6;

const ELLIPSIS = "…";

const graphemes = new Intl.Segmenter(undefined, { granularity: "grapheme" });

export function labelsVisible(mode: LabelMode, rowPx: number): boolean {
  return mode === "on" || (mode === "auto" && rowPx >= LABEL_AUTO_MIN_ROW_PX);
}

export function ribbonsShown(rowPx: number): boolean {
  return rowPx < RIBBON_MAX_ROW_PX;
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

const LABEL_FACE_FAMILY = "IBM Plex Sans Condensed";

export interface LoadedFace {
  family: string;
}

export interface FontLoader {
  load(font: string, text: string): Promise<readonly LoadedFace[]>;
  reportFailure(font: string, error: Error): void;
}

export interface FontStore {
  subscribe(text: string, onChange: () => void): () => void;
  isReady(text: string): boolean;
}

export function labelCharacters(names: Iterable<string>): string {
  const characters = new Set<string>([ELLIPSIS]);

  for (const name of names) {
    for (const character of name) {
      characters.add(character);
    }
  }

  return [...characters].toSorted().join("");
}

export function labelFontStore(loader: FontLoader): FontStore {
  const ready = new Set<string>();
  const loading = new Set<string>();
  const listeners = new Set<() => void>();

  async function waitForFont(text: string): Promise<void> {
    try {
      const faces = await loader.load(LABEL_FONT, text);

      if (!faces.some((face) => face.family.replaceAll(/^["']|["']$/gu, "") === LABEL_FACE_FAMILY)) {
        loader.reportFailure(LABEL_FONT, new Error(`No ${LABEL_FACE_FAMILY} font face covers the label characters`));
      }
    } catch (error) {
      loader.reportFailure(LABEL_FONT, error instanceof Error ? error : new Error(String(error)));
    }

    ready.add(text);

    for (const listener of listeners) {
      listener();
    }
  }

  return {
    subscribe(text, onChange) {
      listeners.add(onChange);

      if (!loading.has(text)) {
        loading.add(text);
        void waitForFont(text);
      }

      return () => {
        listeners.delete(onChange);
      };
    },
    isReady(text) {
      return ready.has(text);
    },
  };
}

const fontStore = labelFontStore({
  load: async (font, text) => document.fonts.load(font, text),
  reportFailure: (font, error) => {
    console.error(`The label font "${font}" did not load, so labels use a fallback font`, error);
  },
});

function fontNotReadyOnServer(): boolean {
  return false;
}

export function useLabelFontReady(text: string): boolean {
  const subscribe = useCallback((onChange: () => void) => fontStore.subscribe(text, onChange), [text]);
  const isReady = useCallback(() => fontStore.isReady(text), [text]);

  return useSyncExternalStore(subscribe, isReady, fontNotReadyOnServer);
}
