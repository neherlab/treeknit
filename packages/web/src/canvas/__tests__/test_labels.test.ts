import type { DrawingRules } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import {
  type FontStore,
  LABEL_FONT,
  labelCharacters,
  labelFontStore,
  labelsVisible,
  ribbonsShown,
  shortenLabel,
} from "../labels";

const RULES: DrawingRules = { labelAutoMinRowPx: 10, linkMinRowPx: 6, labelMaxChars: 40 };

function graphemeCount(text: string): number {
  return Array.from(new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(text)).length;
}

describe("labelsVisible", () => {
  test.each([
    ["auto", 9.99, false],
    ["auto", 10, true],
    ["auto", 30, true],
    ["on", 0.5, true],
    ["on", 30, true],
    ["off", 0.5, false],
    ["off", 30, false],
  ] as const)("mode %s at %f px per row shows labels: %s", (mode, rowPx, visible) => {
    expect(labelsVisible(mode, rowPx, RULES)).toBe(visible);
  });
});

describe("ribbonsShown", () => {
  test.each([
    [0.1, true],
    [5.99, true],
    [6, false],
    [30, false],
  ] as const)("at %f px per row draws ribbons: %s", (rowPx, ribbons) => {
    expect(ribbonsShown(rowPx, RULES)).toBe(ribbons);
  });
});

describe("shortenLabel", () => {
  test("keeps a label of exactly the maximum length", () => {
    const name = "A".repeat(40);

    expect(shortenLabel(name, 40)).toBe(name);
  });

  test("keeps a short strain name unchanged", () => {
    expect(shortenLabel("A/New York/392/2004", 40)).toBe("A/New York/392/2004");
  });

  test("shortens a longer label in the middle to the maximum length", () => {
    const name = "A/Hong Kong/1-0123456789/2004|EPI_ISL_000000|H3N2|2004-01-02";
    const short = shortenLabel(name, 40);

    expect(short).toBe("A/Hong Kong/1-012345…000|H3N2|2004-01-02");
    expect(graphemeCount(short)).toBe(40);
  });

  test("never splits a letter with a combining mark or a surrogate pair", () => {
    const name = `${"e\u0301".repeat(25)}${"𝔸".repeat(25)}`;
    const short = shortenLabel(name, 11);

    expect(short).toBe(`${"e\u0301".repeat(5)}…${"𝔸".repeat(5)}`);
  });
});

describe("labelCharacters", () => {
  test("lists each character of the names once in code unit order, with the ellipsis of shortened labels", () => {
    expect(labelCharacters(["A/Texas", "A/Kyiv/Київ"])).toBe("/AKTaeisvxyКвиї…");
  });

  test("keeps a letter outside the basic plane whole", () => {
    expect(labelCharacters(["𝔸𝔸"])).toBe("…𝔸");
  });
});

describe("labelFontStore", () => {
  const PLEX_FACE = { family: '"IBM Plex Sans Condensed"' };

  test("reports a failed font load with the font and keeps labels drawable with the fallback font", async () => {
    const failure = new Error("network error");
    const reports: [string, Error][] = [];

    const store = labelFontStore({
      load: () => Promise.reject(failure),
      reportFailure: (font, error) => {
        reports.push([font, error]);
      },
    });

    await firstChange(store, "AB");

    expect(reports).toStrictEqual([[LABEL_FONT, failure]]);
    expect(store.isReady("AB")).toBe(true);
  });

  test.each([
    ["no font face", []],
    ["only a face of another family", [{ family: "IBM Plex Sans" }]],
  ])("reports a load that finds %s for the label characters", async (_case, faces) => {
    const reports: string[] = [];

    const store = labelFontStore({
      load: () => Promise.resolve(faces),
      reportFailure: (_font, error) => {
        reports.push(error.message);
      },
    });

    await firstChange(store, "Ж");

    expect(reports).toStrictEqual(["No IBM Plex Sans Condensed font face covers the label characters"]);
    expect(store.isReady("Ж")).toBe(true);
  });

  test("loads the label font once per text, with that text, and reports nothing when it loads", async () => {
    const loaded: [string, string][] = [];
    const reports: Error[] = [];

    const store = labelFontStore({
      load: (font, text) => {
        loaded.push([font, text]);

        return Promise.resolve([PLEX_FACE]);
      },
      reportFailure: (_font, error) => {
        reports.push(error);
      },
    });

    expect(store.isReady("AB")).toBe(false);

    await Promise.all([firstChange(store, "AB"), firstChange(store, "AB")]);

    expect(store.isReady("AB")).toBe(true);
    expect(store.isReady("Ж")).toBe(false);

    await firstChange(store, "Ж");

    expect(loaded).toStrictEqual([
      [LABEL_FONT, "AB"],
      [LABEL_FONT, "Ж"],
    ]);
    expect(reports).toStrictEqual([]);
    expect(store.isReady("Ж")).toBe(true);
  });
});

function firstChange(store: FontStore, text: string): Promise<void> {
  return new Promise((resolve) => {
    store.subscribe(text, () => {
      resolve();
    });
  });
}
