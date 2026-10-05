import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { COPY_FEEDBACK_MS, copyFeedback, type CopyState } from "../copyFeedback";

describe("copyFeedback", () => {
  const states: CopyState[] = [];
  const written: string[] = [];

  function recordState(state: CopyState) {
    states.push(state);
  }

  function writeSucceeds(text: string) {
    written.push(text);

    return Promise.resolve();
  }

  function writeFails() {
    return Promise.reject(new Error("Clipboard access denied"));
  }

  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    states.length = 0;
    written.length = 0;
  });

  test("writes the text to the clipboard and shows copied", async () => {
    await copyFeedback(writeSucceeds, recordState).copy("((A,B),C);");

    expect(written).toStrictEqual(["((A,B),C);"]);
    expect(states).toStrictEqual(["copied"]);
  });

  test("returns to idle two seconds after a copy, not before", async () => {
    await copyFeedback(writeSucceeds, recordState).copy("(A,B);");

    vi.advanceTimersByTime(COPY_FEEDBACK_MS - 1);
    expect(states).toStrictEqual(["copied"]);

    vi.advanceTimersByTime(1);
    expect(states).toStrictEqual(["copied", "idle"]);
  });

  test("shows failed when the clipboard rejects the write, then returns to idle", async () => {
    await copyFeedback(writeFails, recordState).copy("(A,B);");

    expect(states).toStrictEqual(["failed"]);

    vi.advanceTimersByTime(COPY_FEEDBACK_MS);
    expect(states).toStrictEqual(["failed", "idle"]);
  });

  test("restarts the two seconds on a second copy", async () => {
    const feedback = copyFeedback(writeSucceeds, recordState);

    await feedback.copy("(A,B);");
    vi.advanceTimersByTime(COPY_FEEDBACK_MS - 500);
    await feedback.copy("(A,B);");
    vi.advanceTimersByTime(COPY_FEEDBACK_MS - 1);

    expect(states).toStrictEqual(["copied", "copied"]);

    vi.advanceTimersByTime(1);
    expect(states).toStrictEqual(["copied", "copied", "idle"]);
  });

  test("cancels the pending return to idle on dispose", async () => {
    const feedback = copyFeedback(writeSucceeds, recordState);

    await feedback.copy("(A,B);");
    feedback.dispose();
    vi.advanceTimersByTime(COPY_FEEDBACK_MS);

    expect(states).toStrictEqual(["copied"]);
  });

  test("ignores a write that ends after dispose and starts no timer", async () => {
    const pending = Promise.withResolvers<undefined>();
    const feedback = copyFeedback(() => pending.promise, recordState);

    const copied = feedback.copy("(A,B);");
    feedback.dispose();
    pending.resolve(undefined);
    await copied;

    expect(vi.getTimerCount()).toBe(0);
    expect(states).toStrictEqual([]);
  });

  test("shows the result of the latest copy when an earlier copy ends after it", async () => {
    const earlier = Promise.withResolvers<undefined>();
    const later = Promise.withResolvers<undefined>();
    const writes = [earlier.promise, later.promise];
    const feedback = copyFeedback(() => writes.shift() ?? Promise.resolve(), recordState);

    const first = feedback.copy("(A,B);");
    const second = feedback.copy("(A,C);");
    later.reject(new Error("Clipboard access denied"));
    await second;
    earlier.resolve(undefined);
    await first;

    expect(states).toStrictEqual(["failed"]);
  });

  test("ignores an earlier copy that ends before the latest one", async () => {
    const earlier = Promise.withResolvers<undefined>();
    const later = Promise.withResolvers<undefined>();
    const writes = [earlier.promise, later.promise];
    const feedback = copyFeedback(() => writes.shift() ?? Promise.resolve(), recordState);

    const first = feedback.copy("(A,B);");
    const second = feedback.copy("(A,C);");
    earlier.resolve(undefined);
    await first;

    expect(states).toStrictEqual([]);

    later.resolve(undefined);
    await second;

    expect(states).toStrictEqual(["copied"]);
  });
});
