import type { OutputFile } from "@neherlab/treeknit-wasm";

const REVOKE_DELAY_MS = 60_000;

export function saveFile(file: OutputFile): void {
  const url = URL.createObjectURL(new Blob([file.text], { type: file.mediaType }));
  const anchor = document.createElement("a");

  anchor.href = url;
  anchor.download = file.name;
  anchor.click();
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, REVOKE_DELAY_MS);
}
