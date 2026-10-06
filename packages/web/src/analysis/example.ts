import type { ExampleInfo } from "@neherlab/treeknit-wasm";

const REPOSITORY_ROOT = "../../../../";

const TREE_FILES = {
  ...import.meta.glob<string>("../../../../data/*/*.nwk", { query: "?raw", import: "default" }),
  ...import.meta.glob<string>("../../../../fixtures/sim/*/*.nwk", { query: "?raw", import: "default" }),
} satisfies TreeFileReaders;

export async function loadExample(example: ExampleInfo, files: TreeFileReaders = TREE_FILES): Promise<ExampleTree[]> {
  return Promise.all(
    example.trees.map(async ({ file, label, path, newick }) => ({
      fileName: file,
      label,
      newick: newick ?? (await readTreeFile(path, files)),
    })),
  );
}

export function exampleGroups(examples: readonly ExampleInfo[]): ExampleGroup[] {
  return Array.from(
    Map.groupBy(examples, ({ group }) => group),
    ([id, members]) => ({
      id,
      name: members[0]?.groupName ?? id,
      examples: members,
    }),
  );
}

export function exampleTreePaths(files: TreeFileReaders = TREE_FILES): string[] {
  return Object.keys(files)
    .map((key) => key.slice(REPOSITORY_ROOT.length))
    .toSorted();
}

export interface ExampleGroup {
  id: string;
  name: string;
  examples: readonly ExampleInfo[];
}

export interface ExampleTree {
  fileName: string;
  label: string;
  newick: string;
}

export type TreeFileReaders = Record<string, () => Promise<string>>;

async function readTreeFile(path: string | null, files: TreeFileReaders): Promise<string> {
  const read = path === null ? undefined : files[`${REPOSITORY_ROOT}${path}`];

  if (read === undefined) {
    throw new Error(`The example tree file ${path ?? "(none)"} is not part of this build.`);
  }

  return read();
}
