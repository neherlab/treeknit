import type { AnalysisRequest, Settings, TreeText } from "@neherlab/treeknit-wasm";

export function analysisRequest(form: AnalysisForm): AnalysisRequest {
  return {
    trees: form.trees.map(({ label, newick }) => ({ label, newick })),
    settings: form.settings,
  };
}

export function treeLabel(fileName: string): string {
  const dot = fileName.lastIndexOf(".");

  return dot > 0 ? fileName.slice(0, dot) : fileName;
}

export async function readTrees(files: readonly File[]): Promise<TreeText[]> {
  return Promise.all(files.map(async (file) => ({ label: treeLabel(file.name), newick: await file.text() })));
}

export interface AnalysisForm {
  trees: TreeText[];
  settings: Settings;
}
