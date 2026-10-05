import type { Progress } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { PROGRESS_INTERVAL_MS, ProgressThrottle } from "../progressThrottle";

describe("progress throttle", () => {
  test("forwards the first event and then at most one event per interval", () => {
    const { clock, forwarded, throttle } = recording();

    for (const at of [0, 10, 50, PROGRESS_INTERVAL_MS - 1, PROGRESS_INTERVAL_MS, PROGRESS_INTERVAL_MS + 20]) {
      clock.now = at;
      throttle.push(progressAt(at / 1000));
    }

    expect(forwarded.map(({ fraction }) => fraction)).toStrictEqual([0, PROGRESS_INTERVAL_MS / 1000]);
  });

  test("always forwards the done event", () => {
    const { clock, forwarded, throttle } = recording();

    throttle.push(progressAt(0.1));
    clock.now = 1;
    throttle.push({ ...progressAt(1), phase: "done" });

    expect(forwarded.map(({ phase }) => phase)).toStrictEqual(["pairs", "done"]);
  });

  test("forwards a change of phase, round, or pair at once", () => {
    const { clock, forwarded, throttle } = recording();
    const first = progressAt(0.1);

    throttle.push(first);
    clock.now = 1;
    throttle.push({ ...first, pair: 2 });
    clock.now = 2;
    throttle.push({ ...first, pair: 2, round: 2 });
    clock.now = 3;
    throttle.push({ ...first, pair: 2, round: 2, phase: "matching" });
    clock.now = 4;
    throttle.push({ ...first, pair: 2, round: 2, phase: "matching", fraction: 0.2 });

    expect(forwarded.map(({ phase, round, pair }) => [phase, round, pair])).toStrictEqual([
      ["pairs", 1, 1],
      ["pairs", 1, 2],
      ["pairs", 2, 2],
      ["matching", 2, 2],
    ]);
  });

  test("the done event drops a held event, so flush sends nothing after it", () => {
    const { clock, forwarded, throttle } = recording();

    throttle.push(progressAt(0.1));
    clock.now = 5;
    throttle.push(progressAt(0.2));
    clock.now = 6;
    throttle.push({ ...progressAt(1), phase: "done" });
    throttle.flush();

    expect(forwarded.map(({ phase, fraction }) => [phase, fraction])).toStrictEqual([
      ["pairs", 0.1],
      ["done", 1],
    ]);
  });

  test("flush forwards the last held event once", () => {
    const { clock, forwarded, throttle } = recording();

    throttle.push(progressAt(0.1));
    clock.now = 5;
    throttle.push(progressAt(0.2));
    clock.now = 6;
    throttle.push(progressAt(0.3));
    throttle.flush();
    throttle.flush();

    expect(forwarded.map(({ fraction }) => fraction)).toStrictEqual([0.1, 0.3]);
  });

  test("flush forwards nothing when no event is held", () => {
    const { forwarded, throttle } = recording();

    throttle.push(progressAt(0.1));
    throttle.flush();

    expect(forwarded.map(({ fraction }) => fraction)).toStrictEqual([0.1]);
  });
});

function recording() {
  const clock = { now: 0 };
  const forwarded: Progress[] = [];

  const throttle = new ProgressThrottle(
    (progress) => {
      forwarded.push(progress);
    },
    () => clock.now,
  );

  return { clock, forwarded, throttle };
}

function progressAt(fraction: number): Progress {
  return { phase: "pairs", fraction, round: 1, rounds: 1, pair: 1, pairs: 3 };
}
