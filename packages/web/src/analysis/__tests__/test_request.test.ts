import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import { type AnalysisForm, analysisRequest, readTrees, treeLabel } from "../request";

describe("analysisRequest", () => {
  test("sends the trees and the settings of the form", () => {
    const form: AnalysisForm = {
      trees: [
        { label: "ha", newick: "((A,B),C);" },
        { label: "na", newick: "(A,(B,C));" },
      ],
      settings: { gamma: 3, resolve: "none", seed: 7 },
    };

    expect(analysisRequest(form)).toStrictEqual({
      trees: [
        { label: "ha", newick: "((A,B),C);" },
        { label: "na", newick: "(A,(B,C));" },
      ],
      settings: { gamma: 3, resolve: "none", seed: 7 },
    });
  });
});

describe("treeLabel", () => {
  test.each([
    ["ha.nwk", "ha"],
    ["h3n2.ha.tree", "h3n2.ha"],
    ["segment", "segment"],
    [".nwk", ".nwk"],
  ])("labels a tree from %s as %s", (fileName, expected) => {
    expect(treeLabel(fileName)).toBe(expected);
  });

  test("removes exactly the last extension", () => {
    const stem = fc.string({ minLength: 1 });
    const extension = fc.string().filter((text) => !text.includes("."));

    fc.assert(
      fc.property(stem, extension, (name, ext) => {
        expect(treeLabel(`${name}.${ext}`)).toBe(name);
      }),
    );
  });
});

describe("readTrees", () => {
  test("reads each file as one tree labeled by its file name", async () => {
    const files = [new File(["((A,B),C);"], "ha.nwk"), new File(["(A,(B,C));"], "na.nwk")];

    await expect(readTrees(files)).resolves.toStrictEqual([
      { label: "ha", newick: "((A,B),C);" },
      { label: "na", newick: "(A,(B,C));" },
    ]);
  });
});
