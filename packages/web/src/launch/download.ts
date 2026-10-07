import { getErrorMessage } from "react-error-boundary";
import { match } from "ts-pattern";

export type FetchFile = (url: string, init: RequestInit) => Promise<Response>;

export interface DownloadLimits {
  timeoutSeconds: number;
  maxBytes: number;
}

export type DownloadFailure =
  | { kind: "network" }
  | { kind: "status"; status: number; statusText: string }
  | { kind: "timeout" }
  | { kind: "tooLarge" };

export class DownloadError extends Error {
  readonly failure: DownloadFailure;

  constructor(failure: DownloadFailure, options?: ErrorOptions) {
    super(`The download failed: ${failure.kind}`, options);
    this.name = "DownloadError";
    this.failure = failure;
  }
}

export async function download(url: string, limits: DownloadLimits, fetchFile: FetchFile): Promise<Uint8Array> {
  const signal = AbortSignal.timeout(limits.timeoutSeconds * 1000);
  const failed = (cause: unknown) => new DownloadError({ kind: signal.aborted ? "timeout" : "network" }, { cause });
  let response: Response;

  try {
    response = await fetchFile(url, { credentials: "omit", signal });
  } catch (cause) {
    throw failed(cause);
  }

  if (!response.ok) {
    throw new DownloadError({ kind: "status", status: response.status, statusText: response.statusText });
  }

  try {
    return await readLimited(response, limits.maxBytes);
  } catch (cause) {
    throw cause instanceof DownloadError ? cause : failed(cause);
  }
}

export function downloadMessage(url: string, cause: unknown, limits: DownloadLimits): string {
  const place = filePlace(url);

  if (!(cause instanceof DownloadError)) {
    return `Could not read ${place}: ${getErrorMessage(cause) ?? String(cause)}`;
  }

  return match(cause.failure)
    .with(
      { kind: "network" },
      () =>
        `Could not read ${place}. The server may not allow other sites to read it (CORS), the address may be wrong, or the network is down. Download the file and drop it here instead.`,
    )
    .with({ kind: "status" }, ({ status, statusText }) =>
      statusText === ""
        ? `Could not read ${place}: HTTP status ${String(status)}`
        : `Could not read ${place}: ${String(status)} ${statusText}`,
    )
    .with(
      { kind: "timeout" },
      () => `Could not read ${place}: no answer within ${String(limits.timeoutSeconds)} seconds.`,
    )
    .with(
      { kind: "tooLarge" },
      () => `Could not read ${place}: the file is larger than ${String(limits.maxBytes / 1024 / 1024)} MiB.`,
    )
    .exhaustive();
}

export function filePlace(url: string): string {
  const parsed = URL.parse(url);

  if (parsed === null) {
    return url;
  }

  const file = parsed.pathname.split("/").findLast((part) => part !== "");

  return file === undefined ? parsed.host : `${parsed.host}/${safeDecode(file)}`;
}

async function readLimited(response: Response, maxBytes: number): Promise<Uint8Array> {
  const reader = response.body?.getReader();

  if (reader === undefined) {
    const bytes = new Uint8Array(await response.arrayBuffer());

    if (bytes.byteLength > maxBytes) {
      throw new DownloadError({ kind: "tooLarge" });
    }

    return bytes;
  }

  const chunks: Uint8Array<ArrayBuffer>[] = [];
  let total = 0;

  for (let next = await reader.read(); !next.done; next = await reader.read()) {
    total += next.value.byteLength;

    if (total > maxBytes) {
      await reader.cancel();

      throw new DownloadError({ kind: "tooLarge" });
    }

    chunks.push(next.value);
  }

  return new Uint8Array(await new Blob(chunks).arrayBuffer());
}

function safeDecode(text: string): string {
  try {
    return decodeURIComponent(text);
  } catch {
    return text;
  }
}
