import { describe, expect, test } from "vitest";

import {
  type FontStore,
  LABEL_AUTO_MIN_ROW_PX,
  LABEL_FONT,
  LABEL_MAX_LENGTH,
  labelFontStore,
  labelsVisible,
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

describe("labelFontStore", () => {
  test("reports a failed font load with the font and keeps labels drawable with the fallback font", async () => {
    const failure = new Error("network error");
    const reports: [string, Error][] = [];

    const store = labelFontStore({
      load: () => Promise.reject(failure),
      reportFailure: (font, error) => {
        reports.push([font, error]);
      },
    });

    await firstChange(store);

    expect(reports).toStrictEqual([[LABEL_FONT, failure]]);
    expect(store.isReady()).toBe(true);
  });

  test("loads the label font once and reports nothing when it loads", async () => {
    const loaded: string[] = [];
    const reports: Error[] = [];

    const store = labelFontStore({
      load: (font) => {
        loaded.push(font);

        return Promise.resolve();
      },
      reportFailure: (_font, error) => {
        reports.push(error);
      },
    });

    expect(store.isReady()).toBe(false);

    await Promise.all([
      firstChange(store),
      firstChange(store),
    ]);

    expect(loaded).toStrictEqual([LABEL_FONT]);
    expect(reports).toStrictEqual([]);
    expect(store.isReady()).toBe(true);
  });
});

function firstChange(store: FontStore): Promise<void> {
  return new Promise((resolve) => {
    store.subscribe(() => {
      resolve();
    });
  });
}
