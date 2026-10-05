import type { Figure, FileEntry } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { FIGURE_LISTING, FIGURE_MISSING, FIGURE_UNLISTED, figureContent, figureFile, figureMutation } from "../figure";

const SVG = "image/svg+xml";

const PAIR_0: Figure = { kind: "pair", pair: 0 };

const PAIR_1: Figure = { kind: "pair", pair: 1 };

const TWO_TREES: FileEntry[] = [
  { path: "MCCs.json", fileName: "MCCs.json", mediaType: "application/json", size: 120, figure: null },
  {
    path: "tanglegram_ha_na.svg",
    fileName: "tanglegram_ha_na.svg",
    mediaType: SVG,
    size: null,
    figure: { kind: "pair", pair: 0 },
  },
  { path: "ARG/arg.svg", fileName: "arg.svg", mediaType: SVG, size: null, figure: { kind: "arg" } },
];

const THREE_TREES: FileEntry[] = [
  { path: "MCCs.json", fileName: "MCCs.json", mediaType: "application/json", size: 240, figure: null },
  {
    path: "tanglegram_ha_na.svg",
    fileName: "tanglegram_ha_na.svg",
    mediaType: SVG,
    size: null,
    figure: { kind: "pair", pair: 0 },
  },
  {
    path: "tanglegram_ha_pb2.svg",
    fileName: "tanglegram_ha_pb2.svg",
    mediaType: SVG,
    size: null,
    figure: { kind: "pair", pair: 1 },
  },
  {
    path: "tanglegram_na_pb2.svg",
    fileName: "tanglegram_na_pb2.svg",
    mediaType: SVG,
    size: 5_400,
    figure: { kind: "pair", pair: 2 },
  },
];

describe("figure downloads", () => {
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

  test("save the figure under the file name that Rust gives its options, with the media type of its file entry", () => {
    expect(
      figureContent({ fileName: "tanglegram_ha_na_imputed_depth_w800.svg", text: "<svg/>" }, { mediaType: SVG }),
    ).toStrictEqual({ name: "tanglegram_ha_na_imputed_depth_w800.svg", mediaType: SVG, content: "<svg/>" });
  });
});

describe("figure download state", () => {
  const FAILED = new Error("worker stopped");

  test("belongs to the figure whose download started", () => {
    expect(
      figureMutation({ variables: { figure: { kind: "pair", pair: 0 } }, isPending: true, error: null }, PAIR_0),
    ).toStrictEqual({ isPending: true, error: null });
  });

  test("does not show the pending download or the failure of another figure", () => {
    expect([
      figureMutation({ variables: { figure: PAIR_0 }, isPending: true, error: null }, PAIR_1),
      figureMutation({ variables: { figure: PAIR_0 }, isPending: false, error: FAILED }, PAIR_1),
      figureMutation({ variables: { figure: PAIR_0 }, isPending: false, error: FAILED }, { kind: "arg" }),
    ]).toStrictEqual([
      { isPending: false, error: null },
      { isPending: false, error: null },
      { isPending: false, error: null },
    ]);
  });

  test("is idle before any download", () => {
    expect(figureMutation({ variables: undefined, isPending: false, error: null }, PAIR_0)).toStrictEqual({
      isPending: false,
      error: null,
    });
  });
});

function pathOf(files: readonly FileEntry[], figure: Figure): string | undefined {
  const file = figureFile({ data: files, error: null }, figure);

  return "entry" in file ? file.entry.path : undefined;
}
