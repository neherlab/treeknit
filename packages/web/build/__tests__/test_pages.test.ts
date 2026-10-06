import { describe, expect, test } from "vitest";

import { type BundleFile, pageCopies } from "../pages";

const INDEX: BundleFile = { type: "asset", source: "<!doctype html><title>TreeKnit</title>" };

describe("pageCopies", () => {
  test("copies index.html once for each page besides the root", () => {
    const bundle = { "index.html": INDEX, "assets/index-1a2b.js": { type: "chunk" } };

    expect(pageCopies(bundle, ["/", "/help"])).toStrictEqual([{ fileName: "help.html", source: INDEX.source }]);
  });

  test("fails without index.html", () => {
    expect(() => pageCopies({ "assets/index-1a2b.js": { type: "chunk" } }, ["/", "/help"])).toThrow(
      "The build wrote no index.html to copy for the other pages.",
    );
  });
});
