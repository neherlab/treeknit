import { describe, expect, test } from "vitest";

import { LABEL_AUTO_MIN_ROW_PX, LABEL_MAX_LENGTH, labelsVisible, shortenLabel } from "../labels";

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
