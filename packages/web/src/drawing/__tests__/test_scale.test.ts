import { describe, expect, test } from "vitest";

import { shownScaleNotice } from "../scale";

describe("shown scale notice", () => {
  test("names the cladogram that a drawing without branch lengths shows for divergence", () => {
    expect([shownScaleNotice("div", "depth", "pair"), shownScaleNotice("div", "depth", "arg")]).toStrictEqual([
      { tone: "info", text: "Cladogram: a tree of this pair has no branch lengths." },
      { tone: "info", text: "Cladogram: neither tree has branch lengths." },
    ]);
  });

  test("is absent when the drawing shows the requested scale or has not loaded", () => {
    expect([
      shownScaleNotice("div", "div", "pair"),
      shownScaleNotice("depth", "depth", "arg"),
      shownScaleNotice("div", undefined, "pair"),
    ]).toStrictEqual([undefined, undefined, undefined]);
  });
});
