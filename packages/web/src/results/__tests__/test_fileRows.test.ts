import type { FileEntry } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { downloadName, fileRow, fileSizeLabel, MADE_ON_DOWNLOAD } from "../fileRows";

const SVG = "image/svg+xml";

describe("files tab rows", () => {
  test("downloads a file under the last segment of its path", () => {
    expect([downloadName("ARG/arg.nwk"), downloadName("MCCs.json")]).toStrictEqual(["arg.nwk", "MCCs.json"]);
  });

  test("shows the size, or that a figure is made on download", () => {
    expect([fileSizeLabel(512), fileSizeLabel(2_500), fileSizeLabel(null)]).toStrictEqual([
      "512 bytes",
      "2.5 kB",
      MADE_ON_DOWNLOAD,
    ]);
  });

  test("lists the figures of two trees under their names, made on download", () => {
    const files: FileEntry[] = [
      { path: "tanglegram_ha_na.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 0 } },
      { path: "ARG/arg.svg", mediaType: SVG, size: null, figure: { kind: "arg" } },
    ];

    expect(files.map(fileRow).map(({ path, size, name }) => ({ path, size, name }))).toStrictEqual([
      { path: "tanglegram_ha_na.svg", size: MADE_ON_DOWNLOAD, name: "tanglegram_ha_na.svg" },
      { path: "ARG/arg.svg", size: MADE_ON_DOWNLOAD, name: "arg.svg" },
    ]);
  });

  test("lists one tanglegram per pair of three trees, with the size once rendered", () => {
    const files: FileEntry[] = [
      { path: "tanglegram_ha_na.svg", mediaType: SVG, size: 2_500, figure: { kind: "pair", pair: 0 } },
      { path: "tanglegram_ha_pb2.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 1 } },
      { path: "tanglegram_na_pb2.svg", mediaType: SVG, size: null, figure: { kind: "pair", pair: 2 } },
    ];

    expect(files.map(fileRow).map(({ size, name }) => ({ size, name }))).toStrictEqual([
      { size: "2.5 kB", name: "tanglegram_ha_na.svg" },
      { size: MADE_ON_DOWNLOAD, name: "tanglegram_ha_pb2.svg" },
      { size: MADE_ON_DOWNLOAD, name: "tanglegram_na_pb2.svg" },
    ]);
  });
});
