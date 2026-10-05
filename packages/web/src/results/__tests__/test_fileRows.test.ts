import type { FileEntry } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { fileRow, fileSizeLabel, MADE_ON_DOWNLOAD } from "../fileRows";

const SVG = "image/svg+xml";

describe("files tab rows", () => {
  test("shows the size, or that a figure is made on download", () => {
    expect([fileSizeLabel(512), fileSizeLabel(2_500), fileSizeLabel(null)]).toStrictEqual([
      "512 bytes",
      "2.5 kB",
      MADE_ON_DOWNLOAD,
    ]);
  });

  test("keys each row by its path and labels the size of a rendered and an unrendered figure", () => {
    const files: FileEntry[] = [
      {
        path: "tanglegram_ha_na.svg",
        fileName: "tanglegram_ha_na.svg",
        mediaType: SVG,
        size: 2_500,
        figure: { kind: "pair", pair: 0 },
      },
      { path: "ARG/arg.svg", fileName: "arg.svg", mediaType: SVG, size: null, figure: { kind: "arg" } },
    ];

    expect(files.map(fileRow).map(({ id, size }) => ({ id, size }))).toStrictEqual([
      { id: "tanglegram_ha_na.svg", size: "2.5 kB" },
      { id: "ARG/arg.svg", size: MADE_ON_DOWNLOAD },
    ]);
  });
});
