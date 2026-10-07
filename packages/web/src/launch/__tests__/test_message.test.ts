import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { type MessagePortal, MESSAGE_TIMING, type ReceivedMessage, READY_MESSAGE, receiveSession } from "../message";

describe("receiveSession", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test("announces itself to the sender until the sender replies, and takes the session of the reply", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit");

    vi.advanceTimersByTime(MESSAGE_TIMING.intervalMs * 2);
    pair.send(pair.sender, { type: "treeknit:open", session: "{}", run: true });

    await expect(received).resolves.toStrictEqual({ text: "{}", origin: "https://sender.example", run: true });

    vi.advanceTimersByTime(MESSAGE_TIMING.intervalMs);

    expect({ posted: pair.posted, timers: vi.getTimerCount() }).toStrictEqual({
      posted: [READY_MESSAGE, READY_MESSAGE, READY_MESSAGE],
      timers: 0,
    });
  });

  test("ignores replies from other windows and messages of another shape", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit");

    pair.send({ name: "another window" }, { type: "treeknit:open", session: "wrong" });
    pair.send(pair.sender, { type: "something else", session: "wrong" });
    pair.send(pair.sender, { type: "treeknit:open", session: "right" });

    await expect(received).resolves.toStrictEqual({ text: "right", origin: "https://sender.example", run: false });
  });

  test("gives up when the sender does not reply in time", async () => {
    const pair = new WindowPair();
    const received = receiveSession(pair.portal, "page that opened TreeKnit");

    vi.advanceTimersByTime(MESSAGE_TIMING.timeoutMs);

    await expect(received).rejects.toThrow("The page that opened TreeKnit sent no session file within 10 seconds.");
  });

  test("fails at once without a sender window", async () => {
    const portal: MessagePortal = { target: null, listen: () => () => undefined };

    await expect(receiveSession(portal, "page that embeds TreeKnit")).rejects.toThrow(
      "There is no page that embeds TreeKnit to receive a session file from.",
    );
  });
});

class WindowPair {
  readonly posted: unknown[] = [];

  readonly sender = {
    postMessage: (message: unknown) => {
      this.posted.push(message);
    },
  };

  readonly #listeners = new Set<(event: ReceivedMessage) => void>();

  get portal(): MessagePortal {
    return {
      target: this.sender,
      listen: (listener) => {
        this.#listeners.add(listener);

        return () => {
          this.#listeners.delete(listener);
        };
      },
    };
  }

  send(source: unknown, data: unknown): void {
    for (const listener of this.#listeners) {
      listener({ source, origin: "https://sender.example", data });
    }
  }
}
