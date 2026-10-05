import type { DrawingRules } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { type FontStore, LABEL_FONT, labelCharacters, labelFontStore, labelsVisible } from "../labels";

const RULES: DrawingRules = {
  labelAutoMinRowPx: 10,
  linkMinRowPx: 6,
  labelMaxChars: 40,
  marginPx: 16,
  labelGapPx: 6,
  linkZoneShare: 0.2,
  linkZoneMinShare: 0.15,
  tanglegramLabelColumnMaxShare: 0.25,
  argLabelColumnMaxShare: 0.25,
  branchWidthPx: 1.5,
  reassortmentWidthPx: 2,
  linkWidthPx: 1,
  leaderWidthPx: 1,
  leaderOpacity: 0.5,
  markRadiusPx: 3.5,
  markLinePx: 1.5,
  ribbonOpacity: 0.55,
  dashPx: [4, 3],
  dotPx: [1, 3],
};

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

describe("labelCharacters", () => {
  test("lists each character of the labels once in code unit order", () => {
    expect(labelCharacters(["A/Texas", "A/Kyiv/Київ"])).toBe("/AKTaeisvxyКвиї");
  });

  test("keeps a letter outside the basic plane whole", () => {
    expect(labelCharacters(["𝔸𝔸…"])).toBe("…𝔸");
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

  test("loads each character once, with only the characters that are not loaded yet, and reports nothing when it loads", async () => {
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
    expect(store.isReady("BA")).toBe(true);
    expect(store.isReady("ABЖ")).toBe(false);

    await firstChange(store, "ABЖ");

    expect(loaded).toStrictEqual([
      [LABEL_FONT, "AB"],
      [LABEL_FONT, "Ж"],
    ]);
    expect(reports).toStrictEqual([]);
    expect(store.isReady("ABЖ")).toBe(true);
  });

  test("keeps new text ready without a load when its characters are already loaded", async () => {
    let loads = 0;

    const store = labelFontStore({
      load: () => {
        loads += 1;

        return Promise.resolve([PLEX_FACE]);
      },
      reportFailure: () => undefined,
    });

    await firstChange(store, "ABC");

    const unsubscribe = store.subscribe("CA", () => undefined);

    expect(store.isReady("CA")).toBe(true);
    expect(loads).toBe(1);

    unsubscribe();
  });
});

function firstChange(store: FontStore, text: string): Promise<void> {
  return new Promise((resolve) => {
    store.subscribe(text, () => {
      resolve();
    });
  });
}
