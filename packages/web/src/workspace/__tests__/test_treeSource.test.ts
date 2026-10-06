import { describe, expect, test } from "vitest";

import { sourceFileName, sourceOrigin, type TreeSource, treeSourceSchema } from "../treeSource";

describe("sourceFileName", () => {
  test.each<[string, TreeSource, string]>([
    ["a file", { kind: "file", name: "ha.nwk" }, "ha.nwk"],
    ["an example tree", { kind: "example", name: "na.nwk", id: "h3n2-2017" }, "na.nwk"],
    ["a URL", { kind: "url", url: "https://x.org/a/seg%204.nwk?download=1" }, "seg 4.nwk"],
    ["a URL without a file name", { kind: "url", url: "https://x.org/" }, "tree"],
    ["a pasted tree", { kind: "paste" }, "tree"],
  ])("names %s", (_case, source, expected) => {
    expect(sourceFileName(source)).toBe(expected);
  });
});

describe("sourceOrigin", () => {
  test.each<[string, TreeSource, string | undefined]>([
    [
      "the host of a URL",
      { kind: "url", url: "https://raw.githubusercontent.com/o/r/main/ha.nwk" },
      "raw.githubusercontent.com",
    ],
    ["the origin of a message", { kind: "message", origin: "https://nextstrain.org" }, "https://nextstrain.org"],
    ["nothing for a file", { kind: "file", name: "ha.nwk" }, undefined],
  ])("shows %s", (_case, source, expected) => {
    expect(sourceOrigin(source)).toBe(expected);
  });
});

describe("treeSourceSchema", () => {
  test("reads the example source of a record saved before examples had ids", () => {
    expect(treeSourceSchema.parse({ kind: "example", name: "ha.nwk" })).toStrictEqual({
      kind: "example",
      name: "ha.nwk",
    });
  });
});
