import type { Figure, FigureDownload, FigureOptions, FileEntry } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";

import type { FileContent } from "../download";
import type { WorkspaceSearch } from "../workspace/search";

export const FIGURE_LISTING = "Listing the files of this run.";

export const FIGURE_MISSING = "This run has no file for this figure.";

export const FIGURE_UNLISTED = "The files of this run could not be listed.";

export type FigureFile = { entry: FileEntry } | { disabledReason: string };

export function figureOptions({ scale, labels }: Pick<WorkspaceSearch, "scale" | "labels">): FigureOptions {
  return { scale, labels };
}

export interface FigureMutation {
  variables: Pick<FileEntry, "figure"> | undefined;
  isPending: boolean;
  error: Error | null;
}

export function figureMutation(
  { variables, isPending, error }: FigureMutation,
  figure: Figure,
): Pick<FigureMutation, "isPending" | "error"> {
  const current = variables !== undefined && isDeepEqual(variables.figure, figure);

  return { isPending: current && isPending, error: current ? error : null };
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

export function figureContent(
  { fileName, text }: FigureDownload,
  { mediaType }: Pick<FileEntry, "mediaType">,
): FileContent {
  return { name: fileName, mediaType, content: text };
}
