import { describe, expect, test } from "vitest";

import { directoryExamples, EXAMPLE_GROUPS, type TreeFileReaders } from "../example";

describe("directoryExamples", () => {
  test("makes one example per case directory, sorted by directory name", () => {
    const files: TreeFileReaders = {
      "../fixtures/sim/sim_k2_n50_r0.1/tree1.nwk": readsAs("(A,B);"),
      "../fixtures/sim/sim_k2_n100_r0.01/tree2.nwk": readsAs("(A,B);"),
      "../fixtures/sim/sim_k2_n100_r0.01/tree1.nwk": readsAs("(A,B);"),
      "../fixtures/sim/sim_k2_n50_r0.1/tree2.nwk": readsAs("(A,B);"),
    };

    expect(directoryExamples("simulated", files).map(({ id, name }) => ({ id, name }))).toStrictEqual([
      { id: "simulated/sim_k2_n100_r0.01", name: "sim_k2_n100_r0.01" },
      { id: "simulated/sim_k2_n50_r0.1", name: "sim_k2_n50_r0.1" },
    ]);
  });

  test("prefixes each example id with its group", () => {
    const files: TreeFileReaders = {
      "../data/h3n2-2017/ha.nwk": readsAs("(A,B);"),
      "../data/h3n2-2017/na.nwk": readsAs("(A,B);"),
    };

    expect(directoryExamples("real", files).map(({ id, name }) => ({ id, name }))).toStrictEqual([
      { id: "real/h3n2-2017", name: "h3n2-2017" },
    ]);
  });

  test("loads the trees of its directory in file name order, with their file names", async () => {
    const files: TreeFileReaders = {
      "../fixtures/sim/case_b/tree1.nwk": readsAs("(E,F);"),
      "../fixtures/sim/case_a/tree3.nwk": readsAs("(C,D);"),
      "../fixtures/sim/case_a/tree1.nwk": readsAs("((A,B),C);"),
      "../fixtures/sim/case_a/tree2.nwk": readsAs("(A,(B,C));"),
    };

    const [caseA] = directoryExamples("simulated", files);

    await expect(caseA?.load()).resolves.toStrictEqual([
      { fileName: "tree1.nwk", newick: "((A,B),C);" },
      { fileName: "tree2.nwk", newick: "(A,(B,C));" },
      { fileName: "tree3.nwk", newick: "(C,D);" },
    ]);
  });

  test("reads no file until an example is loaded", async () => {
    const read: string[] = [];

    const recordsRead = (path: string) => () => {
      read.push(path);

      return Promise.resolve("(A,B);");
    };

    const [first] = directoryExamples("simulated", {
      "../fixtures/sim/case_a/tree1.nwk": recordsRead("case_a/tree1"),
      "../fixtures/sim/case_a/tree2.nwk": recordsRead("case_a/tree2"),
      "../fixtures/sim/case_b/tree1.nwk": recordsRead("case_b/tree1"),
    });

    expect(read).toStrictEqual([]);

    await first?.load();

    expect(read.toSorted()).toStrictEqual(["case_a/tree1", "case_a/tree2"]);
  });

  test("makes no examples without files", () => {
    expect(directoryExamples("simulated", {})).toStrictEqual([]);
  });
});

describe("example groups", () => {
  test("offers every tree pair of data/ as a real-data example", () => {
    const real = EXAMPLE_GROUPS.find(({ id }) => id === "real");

    expect(real?.examples.map(({ id }) => id)).toStrictEqual([
      "real/h3n2-2012-2018",
      "real/h3n2-2017",
      "real/h3n2-2017-2018",
      "real/h3n2-new-york-1999-2004",
    ]);
  });

  test("loads the small example as the files ha.nwk and na.nwk", async () => {
    const small = EXAMPLE_GROUPS.find(({ id }) => id === "small")?.examples[0];

    await expect(small?.load()).resolves.toStrictEqual([
      { fileName: "ha.nwk", newick: "((A,B),(C,(D,X)));" },
      { fileName: "na.nwk", newick: "((A,(B,X)),(C,D));" },
    ]);
  });

  test("gives groups and examples distinct ids, because they share one menu collection", () => {
    const groupIds = EXAMPLE_GROUPS.map(({ id }) => id);
    const exampleIds = EXAMPLE_GROUPS.flatMap(({ examples }) => examples.map(({ id }) => id));
    const ids = [...groupIds, ...exampleIds];

    expect(new Set(ids).size).toBe(ids.length);
  });
});

function readsAs(newick: string): () => Promise<string> {
  return () => Promise.resolve(newick);
}
