import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { downloadFile } from "../download";

const OBJECT_URL = "blob:treeknit/1";

describe("downloadFile", () => {
  const blobs: Blob[] = [];
  const revoked: string[] = [];

  const anchor = {
    href: "",
    download: "",
    clicks: 0,
    click: () => {
      anchor.clicks += 1;
    },
  };

  beforeEach(() => {
    vi.useFakeTimers();
    vi.stubGlobal("URL", {
      createObjectURL: (blob: Blob) => {
        blobs.push(blob);

        return OBJECT_URL;
      },
      revokeObjectURL: (url: string) => {
        revoked.push(url);
      },
    });
    vi.stubGlobal("document", { createElement: () => anchor });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    blobs.length = 0;
    revoked.length = 0;
    anchor.href = "";
    anchor.download = "";
    anchor.clicks = 0;
  });

  test("saves the content under the file name through one click on an object URL", () => {
    downloadFile({ name: "MCCs.json", mediaType: "application/json", content: "{}" });

    expect(anchor.href).toBe(OBJECT_URL);
    expect(anchor.download).toBe("MCCs.json");
    expect(anchor.clicks).toBe(1);
  });

  test("puts text content into a Blob of the media type", async () => {
    downloadFile({ name: "ha_resolved.nwk", mediaType: "text/plain", content: "((A,B),C);\n" });

    const [blob] = blobs;

    expect(blob?.type).toBe("text/plain");
    await expect(blob?.text()).resolves.toBe("((A,B),C);\n");
  });

  test("puts binary content into the Blob unchanged", async () => {
    const bytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0xff]);

    downloadFile({ name: "treeknit.zip", mediaType: "application/zip", content: bytes });

    const buffer = await blobs[0]?.arrayBuffer();

    expect(buffer === undefined ? undefined : new Uint8Array(buffer)).toStrictEqual(bytes);
  });

  test("revokes the object URL a minute after the click, not before", () => {
    downloadFile({ name: "MCCs.dat", mediaType: "text/plain", content: "X\n" });

    vi.advanceTimersByTime(59_999);
    expect(revoked).toStrictEqual([]);

    vi.advanceTimersByTime(1);
    expect(revoked).toStrictEqual([OBJECT_URL]);
  });
});
