import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { DelayedIndicator, INDICATOR_DELAY_MS, INDICATOR_MINIMUM_MS } from "../delayedIndicator";

describe("delayed indicator", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("never appears for an activity shorter than the delay", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS - 1);
    indicator.set(false);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS + INDICATOR_MINIMUM_MS);

    expect(changes).toStrictEqual([]);
  });

  test("appears once the activity lasts the delay", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS - 1);
    const before = [...changes];
    vi.advanceTimersByTime(1);

    expect({ before, after: changes }).toStrictEqual({ before: [], after: [true] });
  });

  test("stays the minimum time after a short activity ends", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS + 10);
    indicator.set(false);
    vi.advanceTimersByTime(INDICATOR_MINIMUM_MS - 11);
    const before = [...changes];
    vi.advanceTimersByTime(1);

    expect({ before, after: changes }).toStrictEqual({ before: [true], after: [true, false] });
  });

  test("hides at once when the activity ends after the minimum time", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS + INDICATOR_MINIMUM_MS + 100);
    indicator.set(false);

    expect(changes).toStrictEqual([true, false]);
  });

  test("stays visible when the activity resumes during the minimum time", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS);
    indicator.set(false);
    vi.advanceTimersByTime(100);
    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_MINIMUM_MS * 4);

    expect(changes).toStrictEqual([true]);
  });

  test("restarts the delay after a hidden reset", () => {
    const { changes, indicator } = recording();

    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS);
    indicator.reset();
    indicator.set(true);
    vi.advanceTimersByTime(INDICATOR_DELAY_MS - 1);
    const before = [...changes];
    vi.advanceTimersByTime(1);

    expect({ before, after: changes }).toStrictEqual({ before: [true, false], after: [true, false, true] });
  });
});

function recording() {
  const changes: boolean[] = [];

  const indicator = new DelayedIndicator((visible) => {
    changes.push(visible);
  });

  return { changes, indicator };
}
