import type { Figure, FigureOptions, FileEntry } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";

import type { WorkspaceSearch } from "../workspace/search";

export const FIGURE_LISTING = "Listing the files of this run.";

export const FIGURE_MISSING = "This run has no file for this figure.";

export const FIGURE_UNLISTED = "The files of this run could not be listed.";

export type FigureFile = { entry: FileEntry } | { disabledReason: string };

export function figureOptions({ x, labels }: Pick<WorkspaceSearch, "x" | "labels">): FigureOptions {
  return { scale: x, labels };
}

export interface ListedFiles {
  data: readonly FileEntry[] | undefined;
  error: Error | null;
}

export function figureFile({ data, error }: ListedFiles, figure: Figure): FigureFile {
  if (error !== null) {
    return { disabledReason: FIGURE_UNLISTED };
  }

  if (data === undefined) {
    return { disabledReason: FIGURE_LISTING };
  }

  const entry = data.find((file) => isDeepEqual(file.figure, figure));

  return entry === undefined ? { disabledReason: FIGURE_MISSING } : { entry };
}
