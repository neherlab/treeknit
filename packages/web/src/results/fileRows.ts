import { formatBytes } from "../format/bytes";

export const MADE_ON_DOWNLOAD = "made on download";

export const RESULTS_ARCHIVE_NAME = "treeknit_results.zip";

export const ZIP_MEDIA_TYPE = "application/zip";

export function downloadName(path: string): string {
  return path.split("/").at(-1) ?? path;
}

export function fileSizeLabel(size: number | null): string {
  return size === null ? MADE_ON_DOWNLOAD : formatBytes(size);
}
