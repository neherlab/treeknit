import type { Figure, FileEntry } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { FIGURE_LISTING, FIGURE_MISSING, FIGURE_UNLISTED, figureFile, figureOptions } from "../figure";

const SVG = "image/svg+xml";

const TWO_TREES: FileEntry[] = [
  { path: "MCCs.json", mediaType: "application/json", size: 120, figure: null },
  { path: "tanglegram_ha_na.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 0 } },
  { path: "ARG/arg.svg", mediaType: SVG, size: null, figure: { kind: "arg" } },
];

const THREE_TREES: FileEntry[] = [
  { path: "MCCs.json", mediaType: "application/json", size: 240, figure: null },
  { path: "tanglegram_ha_na.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 0 } },
  { path: "tanglegram_ha_pb2.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 1 } },
  { path: "tanglegram_na_pb2.svg", mediaType: SVG, size: 5_400, figure: { kind: "pair", pair: 2 } },
];

describe("figure downloads", () => {
  test("pass the branch scale and the label mode of the URL unchanged", () => {
    expect([
      figureOptions({ x: "div", labels: "auto" }),
      figureOptions({ x: "depth", labels: "off" }),
      figureOptions({ x: "div", labels: "on" }),
    ]).toStrictEqual([
      { scale: "div", labels: "auto" },
      { scale: "depth", labels: "off" },
      { scale: "div", labels: "on" },
    ]);
  });

  test("find the tanglegram and the ARG figure of two trees", () => {
    expect([pathOf(TWO_TREES, { kind: "pair", pair: 0 }), pathOf(TWO_TREES, { kind: "arg" })]).toStrictEqual([
      "tanglegram_ha_na.svg",
      "ARG/arg.svg",
    ]);
  });

  test("find the tanglegram of each pair of three trees, and no ARG figure", () => {
    expect([
      pathOf(THREE_TREES, { kind: "pair", pair: 0 }),
      pathOf(THREE_TREES, { kind: "pair", pair: 1 }),
      pathOf(THREE_TREES, { kind: "pair", pair: 2 }),
      figureFile({ data: THREE_TREES, error: null }, { kind: "arg" }),
    ]).toStrictEqual([
      "tanglegram_ha_na.svg",
      "tanglegram_ha_pb2.svg",
      "tanglegram_na_pb2.svg",
      { disabledReason: FIGURE_MISSING },
    ]);
  });

  test("disable the download while the files are listed or when listing failed", () => {
    expect([
      figureFile({ data: undefined, error: null }, { kind: "arg" }),
      figureFile({ data: undefined, error: new Error("worker stopped") }, { kind: "arg" }),
    ]).toStrictEqual([{ disabledReason: FIGURE_LISTING }, { disabledReason: FIGURE_UNLISTED }]);
  });
});

function pathOf(files: readonly FileEntry[], figure: Figure): string | undefined {
  const file = figureFile({ data: files, error: null }, figure);

  return "entry" in file ? file.entry.path : undefined;
}
