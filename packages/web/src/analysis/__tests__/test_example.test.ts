import type { ExampleInfo } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { exampleGroups, exampleTreePaths, loadExample, type TreeFileReaders } from "../example";

const SMALL: ExampleInfo = {
  id: "5-leaves",
  name: "5 leaves",
  group: "small",
  groupName: "Small",
  trees: [
    { file: "ha.nwk", label: "ha", path: null, newick: "((A,B),(C,(D,X)));" },
    { file: "na.nwk", label: "na", path: null, newick: "((A,(B,X)),(C,D));" },
  ],
};

const REAL: ExampleInfo = {
  id: "h3n2-2017",
  name: "h3n2-2017",
  group: "real",
  groupName: "Real data (influenza A/H3N2)",
  trees: [
    { file: "ha.nwk", label: "ha", path: "data/h3n2-2017/ha.nwk", newick: null },
    { file: "na.nwk", label: "na", path: "data/h3n2-2017/na.nwk", newick: null },
  ],
};

describe("loadExample", () => {
  test("gives the inline trees of an example with their file names and labels", async () => {
    await expect(loadExample(SMALL, {})).resolves.toStrictEqual([
      { fileName: "ha.nwk", label: "ha", newick: "((A,B),(C,(D,X)));" },
      { fileName: "na.nwk", label: "na", newick: "((A,(B,X)),(C,D));" },
    ]);
  });

  test("reads the tree files of an example by their repository paths, and no other file", async () => {
    const read: string[] = [];

    const recordsRead = (path: string) => () => {
      read.push(path);

      return Promise.resolve(`(${path});`);
    };

    const files: TreeFileReaders = {
      "../../../../data/h3n2-2017/ha.nwk": recordsRead("ha"),
      "../../../../data/h3n2-2017/na.nwk": recordsRead("na"),
      "../../../../data/h3n2-2012-2018/ha.nwk": recordsRead("other"),
    };

    const trees = await loadExample(REAL, files);

    expect({ trees: trees.map(({ newick }) => newick), read: read.toSorted() }).toStrictEqual({
      trees: ["(ha);", "(na);"],
      read: ["ha", "na"],
    });
  });

  test("fails for a tree file that the build does not hold", async () => {
    await expect(loadExample(REAL, {})).rejects.toThrow(
      "The example tree file data/h3n2-2017/ha.nwk is not part of this build.",
    );
  });
});

describe("exampleGroups", () => {
  test("groups the examples in their order under the titles of the catalog", () => {
    const groups = exampleGroups([SMALL, REAL, { ...REAL, id: "h3n2-2012-2018" }]);

    expect(groups.map(({ id, name, examples }) => ({ id, name, ids: examples.map((e) => e.id) }))).toStrictEqual([
      { id: "small", name: "Small", ids: ["5-leaves"] },
      { id: "real", name: "Real data (influenza A/H3N2)", ids: ["h3n2-2017", "h3n2-2012-2018"] },
    ]);
  });
});

describe("exampleTreePaths", () => {
  test("bundles the tree files of data/ and fixtures/sim/, which the Rust catalog lists one by one", () => {
    const paths = exampleTreePaths();

    expect([
      paths.includes("data/h3n2-2017/ha.nwk"),
      paths.includes("data/h3n2-2k-4-segments/pb2.nwk"),
      paths.includes("fixtures/sim/sim_k3_n50_r0.1/tree3.nwk"),
      paths.every((path) => /^(data|fixtures\/sim)\/[^/]+\/[^/]+\.nwk$/u.test(path)),
    ]).toStrictEqual([true, true, true, true]);
  });
});
