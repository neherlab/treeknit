const REVOKE_DELAY_MS = 60_000;

export interface DownloadFile {
  name: string;
  mediaType: string;
  content: BlobPart;
}

export function downloadFile(file: DownloadFile): void {
  const url = URL.createObjectURL(new Blob([file.content], { type: file.mediaType }));
  const anchor = document.createElement("a");

  anchor.href = url;
  anchor.download = file.name;
  anchor.click();
  setTimeout(() => {
    URL.revokeObjectURL(url);
  }, REVOKE_DELAY_MS);
}
