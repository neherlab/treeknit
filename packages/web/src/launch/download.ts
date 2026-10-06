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
    return `Could not read ${place}: ${cause instanceof Error ? cause.message : String(cause)}`;
  }

  const { failure } = cause;

  switch (failure.kind) {
    case "network": {
      return `Could not read ${place}. The server may not allow other sites to read it (CORS), the address may be wrong, or the network is down. Download the file and drop it here instead.`;
    }

    case "status": {
      const status =
        failure.statusText === "" ? `HTTP status ${String(failure.status)}` : `${String(failure.status)} ${failure.statusText}`;

      return `Could not read ${place}: ${status}`;
    }

    case "timeout": {
      return `Could not read ${place}: no answer within ${String(limits.timeoutSeconds)} seconds.`;
    }

    case "tooLarge": {
      return `Could not read ${place}: the file is larger than ${String(limits.maxBytes / 1024 / 1024)} MiB.`;
    }
  }
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

  const chunks: Uint8Array[] = [];
  let total = 0;

  for (let next = await reader.read(); !next.done; next = await reader.read()) {
    total += next.value.byteLength;

    if (total > maxBytes) {
      await reader.cancel();

      throw new DownloadError({ kind: "tooLarge" });
    }

    chunks.push(next.value);
  }

  const bytes = new Uint8Array(total);
  let offset = 0;

  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }

  return bytes;
}

function safeDecode(text: string): string {
  try {
    return decodeURIComponent(text);
  } catch {
    return text;
  }
}
