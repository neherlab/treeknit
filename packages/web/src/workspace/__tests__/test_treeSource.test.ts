import type { UrlPlace } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import {
  restoredSources,
  sourceFileName,
  sourceOrigin,
  storedTreeSourceSchema,
  type TreeSource,
  treeSourceSchema,
} from "../treeSource";

const URL_SOURCE: TreeSource = { kind: "url", url: "https://x.org/a/seg%204.nwk", name: "seg 4.nwk", host: "x.org" };

describe("sourceFileName", () => {
  test.each<[string, TreeSource, string]>([
    ["a file", { kind: "file", name: "ha.nwk" }, "ha.nwk"],
    ["an example tree", { kind: "example", name: "na.nwk", id: "h3n2-2017" }, "na.nwk"],
    ["a URL by the file name stored with it", URL_SOURCE, "seg 4.nwk"],
    ["a pasted tree with no name, which the label rules of Rust name", { kind: "paste" }, ""],
  ])("names %s", (_case, source, expected) => {
    expect(sourceFileName(source)).toBe(expected);
  });
});

describe("sourceOrigin", () => {
  test.each<[string, TreeSource, string | undefined]>([
    ["the host stored with a URL", URL_SOURCE, "x.org"],
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

describe("restoredSources", () => {
  test("fills the place of a URL source of a record saved before URL sources stored it, and keeps other sources", async () => {
    const stored = storedTreeSourceSchema.array().parse([
      { kind: "url", url: "https://x.org/a/seg%204.nwk" },
      { kind: "file", name: "ha.nwk" },
    ]);

    const asked: string[] = [];

    const place = (url: string): Promise<UrlPlace> => {
      asked.push(url);

      return Promise.resolve({ host: "x.org", fileName: "seg 4.nwk" });
    };

    expect({ restored: await restoredSources(stored, place), asked }).toStrictEqual({
      restored: [URL_SOURCE, { kind: "file", name: "ha.nwk" }],
      asked: ["https://x.org/a/seg%204.nwk"],
    });
  });
});
