import { transferHandlers } from "comlink";
import { describe, expect, test } from "vitest";

import { NO_PANIC_TEXT_MESSAGE, panicTextHandler, PendingPanicText, withPanicText } from "../panicText";

const PANIC_TEXT = "panicked at packages/treeknit-io/src/run.rs:1:1:\nindex out of bounds";

describe("panic text", () => {
  test("replaces the message of a trap with the pending panic text", () => {
    const trap = new WebAssembly.RuntimeError("unreachable");

    const error = withPanicText(trap, PANIC_TEXT);

    expect(error).toBeInstanceOf(WebAssembly.RuntimeError);
    expect(error).toMatchObject({ name: "RuntimeError", message: PANIC_TEXT, cause: trap });
  });

  test("explains a trap without panic text and keeps the engine message", () => {
    const trap = new WebAssembly.RuntimeError("unreachable");

    const error = withPanicText(trap, undefined);

    expect(error).toBeInstanceOf(WebAssembly.RuntimeError);
    expect(error).toMatchObject({ message: `${NO_PANIC_TEXT_MESSAGE} (unreachable)`, cause: trap });
  });

  test("takes the pending text once", () => {
    const pending = new PendingPanicText();

    pending.record("first");
    pending.record(PANIC_TEXT);

    expect([pending.take(), pending.take()]).toStrictEqual([PANIC_TEXT, undefined]);
  });

  test("serializes a thrown trap with the pending panic text and clears it", () => {
    const { handler, pending } = handlerWith(PANIC_TEXT);

    const [serialized] = handler.serialize({ value: new WebAssembly.RuntimeError("unreachable") });

    expect(serialized).toMatchObject({ isError: true, value: { name: "RuntimeError", message: PANIC_TEXT } });
    expect(pending.take()).toBeUndefined();
  });

  test.each([
    ["an error", new Error("invalid request")],
    ["a range error", new RangeError("Maximum call stack size exceeded")],
    ["a string", "stopped"],
  ])("serializes %s like comlink and clears the pending text", (_name, value) => {
    const { handler, original, pending } = handlerWith(PANIC_TEXT);

    expect(handler.serialize({ value })).toStrictEqual(original.serialize({ value }));
    expect(pending.take()).toBeUndefined();
  });
});

function handlerWith(text: string) {
  const original = transferHandlers.get("throw");

  if (original === undefined) {
    throw new Error("comlink has no transfer handler for thrown values.");
  }

  const pending = new PendingPanicText();
  pending.record(text);

  return { handler: panicTextHandler(original, pending), original, pending };
}
