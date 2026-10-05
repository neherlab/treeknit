import { describe, expect, test } from "vitest";

import {
  type FontStore,
  LABEL_AUTO_MIN_ROW_PX,
  LABEL_FONT,
  LABEL_MAX_LENGTH,
  labelCharacters,
  labelFontStore,
  labelsVisible,
  RIBBON_MAX_ROW_PX,
  ribbonsShown,
  shortenLabel,
} from "../labels";

function graphemeCount(text: string): number {
  return Array.from(new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(text)).length;
}

describe("labelsVisible", () => {
  test.each([
    ["auto", 9.99, false],
    ["auto", LABEL_AUTO_MIN_ROW_PX, true],
    ["auto", 30, true],
    ["on", 0.5, true],
    ["on", 30, true],
    ["off", 0.5, false],
    ["off", 30, false],
  ] as const)("mode %s at %f px per row shows labels: %s", (mode, rowPx, visible) => {
    expect(labelsVisible(mode, rowPx)).toBe(visible);
  });
});

describe("ribbonsShown", () => {
  test.each([
    [0.1, true],
    [5.99, true],
    [RIBBON_MAX_ROW_PX, false],
    [30, false],
  ] as const)("at %f px per row draws ribbons: %s", (rowPx, ribbons) => {
    expect(ribbonsShown(rowPx)).toBe(ribbons);
  });
});

describe("shortenLabel", () => {
  test("keeps a label of exactly the maximum length", () => {
    const name = "A".repeat(LABEL_MAX_LENGTH);

    expect(shortenLabel(name)).toBe(name);
  });

  test("keeps a short strain name unchanged", () => {
    expect(shortenLabel("A/New York/392/2004")).toBe("A/New York/392/2004");
  });

  test("shortens a longer label in the middle to the maximum length", () => {
    const name = "A/Hong Kong/1-0123456789/2004|EPI_ISL_000000|H3N2|2004-01-02";
    const short = shortenLabel(name);

    expect(short).toBe("A/Hong Kong/1-012345…000|H3N2|2004-01-02");
    expect(graphemeCount(short)).toBe(LABEL_MAX_LENGTH);
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
