import { describe, expect, test } from "vitest";

import { type SizedElement, type SizeObserver, watchSize } from "../useMeasuredSize";
import type { MeasuredSize } from "../viewState";

interface FakeObserver extends SizeObserver<SizedElement> {
  observed: SizedElement[];
  disconnected: boolean;
  resize(): void;
}

function fakeObserver() {
  const created: FakeObserver[] = [];

  return {
    created,
    create: (measure: () => void): FakeObserver => {
      const observer: FakeObserver = {
        observed: [],
        disconnected: false,
        observe(element) {
          observer.observed.push(element);
        },
        disconnect() {
          observer.disconnected = true;
        },
        resize() {
          measure();
        },
      };

      created.push(observer);

      return observer;
    },
  };
}

describe("watchSize", () => {
  test("measures the canvas and the area at once, and again on each resize", () => {
    const area = { clientWidth: 1000, clientHeight: 700 };
    const canvas = { clientWidth: 880, clientHeight: 640 };
    const measured: MeasuredSize[] = [];
    const { create, created } = fakeObserver();

    watchSize(
      area,
      canvas,
      (size) => {
        measured.push(size);
      },
      create,
    );
    canvas.clientHeight = 0;
    created[0]?.resize();

    expect({ observed: created[0]?.observed, measured }).toStrictEqual({
      observed: [area, canvas],
      measured: [
        { canvas: { width: 880, height: 640 }, areaWidth: 1000 },
        { canvas: { width: 880, height: 0 }, areaWidth: 1000 },
      ],
    });
  });

  test("stops observing when the watch ends", () => {
    const { create, created } = fakeObserver();
    const stop = watchSize({ clientWidth: 1, clientHeight: 1 }, { clientWidth: 1, clientHeight: 1 }, () => {}, create);

    stop();

    expect(created.map(({ disconnected }) => disconnected)).toStrictEqual([true]);
  });

  test("reports nothing while an element is missing", () => {
    const measured: MeasuredSize[] = [];
    const { create, created } = fakeObserver();

    watchSize(
      null,
      { clientWidth: 880, clientHeight: 640 },
      (size) => {
        measured.push(size);
      },
      create,
    );
    created[0]?.resize();

    expect({ observed: created[0]?.observed.length, measured }).toStrictEqual({ observed: 1, measured: [] });
  });
});
