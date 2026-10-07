import type { ComponentType } from "react";
import { describe, expect, test } from "vitest";

import { DrawingCodeError, lazyCanvas, loadDrawingCode } from "../lazyCanvas";

const Canvas: ComponentType<object> = () => null;

const MODULE = { default: Canvas };

const FAILED_FETCH = new TypeError("Failed to fetch dynamically imported module");

describe("lazyCanvas", () => {
  test("keeps one lazy component across mounts until a reload replaces it and tells the subscribers", () => {
    const canvas = lazyCanvas(loaded);
    const notified = { count: 0 };
    const first = canvas.state.getState().component;

    const unsubscribe = canvas.state.subscribe(() => {
      notified.count += 1;
    });

    const remounted = canvas.state.getState().component;

    canvas.reload();
    unsubscribe();
    canvas.reload();

    expect({
      same: remounted === first,
      replaced: canvas.state.getState().component !== first,
      notified: notified.count,
    }).toStrictEqual({ same: true, replaced: true, notified: 1 });
  });
});

describe("loadDrawingCode", () => {
  test("marks a failed download of the drawing code", async () => {
    const load = loadDrawingCode<object>(failed);

    await expect(load()).rejects.toStrictEqual(new DrawingCodeError(FAILED_FETCH));
  });

  test("passes a loaded module through", async () => {
    await expect(loadDrawingCode(loaded)()).resolves.toBe(MODULE);
  });
});

async function loaded(): Promise<typeof MODULE> {
  await Promise.resolve();

  return MODULE;
}

async function failed(): Promise<typeof MODULE> {
  await Promise.resolve();

  throw FAILED_FETCH;
}
