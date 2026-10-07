import type { FileEntry } from "@neherlab/treeknit-wasm";

import { formatBytes } from "../format/bytes";

export const MADE_ON_DOWNLOAD = "made on download";

export const PLACEHOLDER_LABELS = ["<a>", "<b>"] as const;

export function fileSizeLabel(size: number | null): string {
  return size === null ? MADE_ON_DOWNLOAD : formatBytes(size);
}

export interface FileRow {
  id: string;
  size: string;
  file: FileEntry;
}

export function fileRow(file: FileEntry): FileRow {
  return { id: file.path, size: fileSizeLabel(file.size), file };
}
