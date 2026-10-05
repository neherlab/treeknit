import type { DrawingRules, LabelMode } from "@neherlab/treeknit-wasm";
import { useCallback, useSyncExternalStore } from "react";

export const LABEL_FONT_FAMILY = '"IBM Plex Sans Condensed", "IBM Plex Sans", sans-serif';

export const LABEL_FONT_WEIGHT = 400;

export const LABEL_FONT_SIZE_PX = 12;

const ELLIPSIS = "…";

const graphemes = new Intl.Segmenter(undefined, { granularity: "grapheme" });

export function labelsVisible(mode: LabelMode, rowPx: number, rules: DrawingRules): boolean {
  return mode === "on" || (mode === "auto" && rowPx >= rules.labelAutoMinRowPx);
}

export function shortenLabel(name: string, maxLength: number): string {
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
  const loaded = new Set<string>();
  const loading = new Set<string>();
  const listeners = new Set<() => void>();

  async function waitForFont(characters: readonly string[]): Promise<void> {
    try {
      const faces = await loader.load(LABEL_FONT, characters.join(""));

      if (!faces.some((face) => face.family.replaceAll(/^["']|["']$/gu, "") === LABEL_FACE_FAMILY)) {
        loader.reportFailure(LABEL_FONT, new Error(`No ${LABEL_FACE_FAMILY} font face covers the label characters`));
      }
    } catch (error) {
      loader.reportFailure(LABEL_FONT, error instanceof Error ? error : new Error(String(error)));
    }

    for (const character of characters) {
      loading.delete(character);
      loaded.add(character);
    }

    for (const listener of listeners) {
      listener();
    }
  }

  return {
    subscribe(text, onChange) {
      listeners.add(onChange);

      const missing = [...new Set(text)].filter((character) => !loaded.has(character) && !loading.has(character));

      if (missing.length > 0) {
        for (const character of missing) {
          loading.add(character);
        }

        void waitForFont(missing);
      }

      return () => {
        listeners.delete(onChange);
      };
    },
    isReady(text) {
      for (const character of text) {
        if (!loaded.has(character)) {
          return false;
        }
      }

      return true;
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
