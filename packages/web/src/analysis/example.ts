import type { TreeText } from "@neherlab/treeknit-wasm";
import { entries, groupBy, map, pipe, sortBy } from "remeda";

import { treeLabel } from "./request";

const REAL_TREE_FILES = import.meta.glob<string>("../../../../data/*/*.nwk", {
  query: "?raw",
  import: "default",
});

const SIMULATED_TREE_FILES = import.meta.glob<string>("../../../../fixtures/sim/*/*.nwk", {
  query: "?raw",
  import: "default",
});

export const EXAMPLE_GROUPS: readonly ExampleGroup[] = [
  {
    id: "small",
    name: "Small",
    examples: [{ id: "small", name: "5 leaves", load: () => Promise.resolve(smallTrees()) }],
  },
  {
    id: "real",
    name: "Real data (influenza A/H3N2)",
    examples: directoryExamples("real", REAL_TREE_FILES),
  },
  {
    id: "simulated",
    name: "Simulated (ARGTools)",
    examples: directoryExamples("simulated", SIMULATED_TREE_FILES),
  },
];

export function directoryExamples(group: string, files: TreeFileReaders): Example[] {
  return pipe(
    entries(files),
    groupBy(([path]) => pathParts(path).directory),
    entries(),
    sortBy(([directory]) => directory),
    map(([directory, treeFiles]) => ({
      id: `${group}/${directory}`,
      name: directory,
      load: async () => readTreeFiles(treeFiles),
    })),
  );
}

export interface ExampleGroup {
  id: string;
  name: string;
  examples: readonly Example[];
}

export interface Example {
  id: string;
  name: string;
  load: () => Promise<TreeText[]>;
}

export type TreeFileReaders = Record<string, () => Promise<string>>;

function smallTrees(): TreeText[] {
  return [
    { label: "ha", newick: "((A,B),(C,(D,X)));" },
    { label: "na", newick: "((A,(B,X)),(C,D));" },
  ];
}

async function readTreeFiles(treeFiles: readonly (readonly [string, () => Promise<string>])[]): Promise<TreeText[]> {
  return Promise.all(
    sortBy(treeFiles, ([path]) => path).map(async ([path, read]) => ({
      label: treeLabel(pathParts(path).file),
      newick: await read(),
    })),
  );
}

function pathParts(path: string) {
  const parts = path.split("/");

  return { directory: parts.at(-2) ?? "", file: parts.at(-1) ?? path };
}
