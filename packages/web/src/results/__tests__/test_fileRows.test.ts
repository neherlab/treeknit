import { describe, expect, test } from "vitest";

import { downloadName, fileSizeLabel, MADE_ON_DOWNLOAD } from "../fileRows";

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
});
